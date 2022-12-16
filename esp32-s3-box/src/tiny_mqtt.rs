// Source: https://github.com/bjoernQ/esp32-rust-nostd-temperature-logger/blob/main/src/tiny_mqtt.rs
use esp_println::println;
use esp_wifi::{
    compat::queue::SimpleQueue,
    wifi_interface::{timestamp, WifiError},
};
use mqttrust::{
    encoding::v4::{decode_slice, encode_slice, Connect, Pid, Protocol},
    Mqtt, MqttError, Packet, Publish, QoS, Subscribe, SubscribeTopic,
};
use smoltcp::{
    iface::SocketHandle,
    socket::{Socket, TcpSocket},
    wire::Ipv4Address,
};

#[derive(Debug)]
pub enum TinyMqttError {
    MqttError(MqttError),
    Error(smoltcp::Error),
    WifiError(WifiError),
}

impl From<MqttError> for TinyMqttError {
    fn from(e: MqttError) -> Self {
        TinyMqttError::MqttError(e)
    }
}

impl From<smoltcp::Error> for TinyMqttError {
    fn from(e: smoltcp::Error) -> Self {
        TinyMqttError::Error(e)
    }
}

impl From<WifiError> for TinyMqttError {
    fn from(e: WifiError) -> Self {
        TinyMqttError::WifiError(e)
    }
}

pub struct PacketBuffer {
    bytes: [u8; 1024],
}

impl PacketBuffer {
    pub fn new(packet: Packet<'_>) -> PacketBuffer {
        let mut buf = [0u8; 1024];
        encode_slice(&packet, &mut buf).ok();
        PacketBuffer { bytes: buf }
    }

    pub fn parsed(&self) -> Packet<'_> {
        // this might panic: "InvalidPid(0)" when I send s.th with QoS > 0
        decode_slice(&self.bytes).unwrap().unwrap()
    }
}

pub struct TinyMqtt<'a> {
    client_id: &'a str,
    interface: esp_wifi::wifi_interface::Wifi<'a>,
    queue: core::cell::RefCell<SimpleQueue<(usize, [u8; 1024]), 10>>,
    recv_buffer: [u8; 1024],
    recv_index: usize,
    recv_queue: core::cell::RefCell<SimpleQueue<PacketBuffer, 10>>,
    timeout_secs: u16,
    last_sent_millis: u64,
    current_millis_fn: fn() -> u64,
    receive_callback: Option<&'a dyn Fn(&str, &[u8])>,
    mqtt_socket_handle: SocketHandle,
    local_port: u16,
}

impl<'a> TinyMqtt<'a> {
    pub fn new(
        client_id: &'a str,
        mut interface: esp_wifi::wifi_interface::Wifi<'a>,
        current_millis_fn: fn() -> u64,
        receive_callback: Option<&'a dyn Fn(&str, &[u8])>,
    ) -> TinyMqtt<'a> {
        let mut mqtt_socket_handle: Option<SocketHandle> = None;

        for (handle, socket) in interface.network_interface().sockets_mut() {
            match socket {
                Socket::Tcp(_) => {
                    mqtt_socket_handle = Some(handle);
                    break;
                }
                _ => {}
            }
        }

        let res = TinyMqtt {
            client_id,
            interface,
            queue: core::cell::RefCell::new(SimpleQueue::new()),
            recv_buffer: [0u8; 1024],
            recv_index: 0,
            recv_queue: core::cell::RefCell::new(SimpleQueue::new()),
            timeout_secs: 0,
            last_sent_millis: 0,
            current_millis_fn,
            receive_callback,
            mqtt_socket_handle: mqtt_socket_handle.unwrap(),
            local_port: 41000,
        };

        res
    }

    pub fn connect(
        &mut self,
        addr: Ipv4Address,
        port: u16,
        keep_alive_secs: u16,
        username: Option<&'a str>,
        password: Option<&'a [u8]>,
    ) -> Result<(), TinyMqttError> {
        self.timeout_secs = keep_alive_secs;

        {
            let (sock, cx) = self
                .interface
                .network_interface()
                .get_socket_and_context::<TcpSocket>(self.mqtt_socket_handle);
            let remote_endpoint = (addr, port);
            sock.connect(cx, remote_endpoint, self.local_port)?;
            self.local_port += 1;
            if self.local_port == 65535 {
                self.local_port = 41000;
            }
        }

        loop {
            let sock = self
                .interface
                .network_interface()
                .get_socket::<TcpSocket>(self.mqtt_socket_handle);
            if sock.can_send() {
                break;
            }
            self.work();
        }

        let connect = Packet::Connect(Connect {
            protocol: Protocol::MQTT311,
            keep_alive: keep_alive_secs,
            client_id: self.client_id(),
            clean_session: true,
            last_will: None,
            username,
            password,
        });

        self.send(connect)?;

        // wait for CONACK
        loop {
            self.poll_internal(false)?;
            if let Some(data) = self.recv_queue.borrow_mut().dequeue() {
                if let Packet::Connack(_) = data.parsed() {
                    break;
                }
            }
        }

        self.last_sent_millis = (self.current_millis_fn)();

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), TinyMqttError> {
        self.interface
            .network_interface()
            .get_socket::<TcpSocket>(self.mqtt_socket_handle)
            .abort();
        Ok(())
    }

    pub fn publish_with_pid(
        &self,
        pid: Option<Pid>,
        topic_name: &str,
        payload: &[u8],
        qos: QoS,
    ) -> Result<(), MqttError> {
        let packet = Packet::Publish(Publish {
            dup: false,
            qos,
            pid: None,
            retain: false,
            topic_name,
            payload,
        });

        let mut buf = [0u8; 1024];
        let len = encode_slice(&packet, &mut buf).unwrap();

        // encode_slice doesn't fill in the PID for publish packets
        if pid.is_some() {
            let pid: u16 = pid.unwrap().into();
            let idx = len - payload.len() - 2;
            buf[idx + 0] = ((pid & 0xff00) >> 8) as u8;
            buf[idx + 1] = (pid & 0xff) as u8;
        }

        self.queue.borrow_mut().enqueue((len, buf));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn subscribe<'b: 'a>(
        &self,
        _pid: Option<Pid>,
        topics: &[SubscribeTopic<'_>],
    ) -> Result<(), MqttError> {
        let subscribe = Subscribe::new(topics);
        let packet = Packet::Subscribe(subscribe);

        self.send(packet)?;

        Ok(())
    }

    pub fn poll(&mut self) -> Result<(), TinyMqttError> {
        self.poll_internal(true)
    }

    fn poll_internal(&mut self, drain_receive_queue: bool) -> Result<(), TinyMqttError> {
        let time = (self.current_millis_fn)();

        if time > self.last_sent_millis + ((self.timeout_secs as u64 / 2) * 1000) {
            // ping
            self.send(Packet::Pingreq)?;
            self.last_sent_millis = (self.current_millis_fn)();
        }

        self.receive_internal()?;
        self.send_internal()?;

        // just drain the received packets for now
        if drain_receive_queue {
            while let Some(received) = self.recv_queue.borrow_mut().dequeue() {
                if let Packet::Publish(publish) = received.parsed() {
                    if let Some(callback) = self.receive_callback {
                        callback(publish.topic_name, publish.payload);
                    }
                }
            }
        }

        Ok(())
    }

    fn receive_internal(&mut self) -> Result<(), TinyMqttError> {
        loop {
            self.work();

            let mut buffer = [0u8; 1024];
            let socket = self
                .interface
                .network_interface()
                .get_socket::<TcpSocket>(self.mqtt_socket_handle);

            match socket.recv_slice(&mut buffer) {
                Ok(len) => {
                    if len > 0 {
                        println!("got {} bytes: {:02x?}", len, &buffer[..len]);
                    }

                    self.recv_buffer[self.recv_index..][..len].copy_from_slice(&buffer[..len]);
                    self.recv_index += len;

                    let data = self.recv_buffer[..len].as_ref();
                    let packet = decode_slice(data);

                    if let Ok(Some(packet)) = packet {
                        println!("{:?}", packet);
                        self.recv_index = 0;
                        self.recv_queue
                            .borrow_mut()
                            .enqueue(PacketBuffer::new(packet));
                    }

                    if len == 0 {
                        return Ok(());
                    }
                }
                Err(e) => {
                    println!("recv error {:?}", e);
                    return Err(e.into());
                }
            }
        }
    }

    fn send_internal(&mut self) -> Result<(), TinyMqttError> {
        loop {
            let dq = self.queue.borrow_mut().dequeue();
            match dq {
                Some((len, buffer)) => loop {
                    self.work();

                    println!("try sending a buffer, len = {}", len);

                    let socket = self
                        .interface
                        .network_interface()
                        .get_socket::<TcpSocket>(self.mqtt_socket_handle);

                    if socket.send_slice(&buffer[..len]).is_ok() {
                        println!("fine");
                        return Ok(());
                    }
                },
                None => return Ok(()),
            }
        }
    }

    fn work(&mut self) {
        loop {
            self.interface.poll_dhcp().ok();
            if let Ok(false) = self.interface.network_interface().poll(timestamp()) {
                break;
            }
        }
    }
}

impl<'a> Mqtt for TinyMqtt<'a> {
    fn send(&self, packet: mqttrust::Packet<'_>) -> Result<(), mqttrust::MqttError> {
        let mut buf = [0u8; 1024];
        let len = encode_slice(&packet, &mut buf).unwrap();

        self.queue.borrow_mut().enqueue((len, buf));
        Ok(())
    }

    fn client_id(&self) -> &str {
        self.client_id
    }
}
