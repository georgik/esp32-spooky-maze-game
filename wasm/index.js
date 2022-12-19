var keyUp;
const rust = import('./pkg')
  .then(m => {
    let universe = m.Universe.new();
    var motion_request = 0;

    document.getElementById('button-up').addEventListener('click', () => {
        universe.move_up();
    });
    document.getElementById('button-down').addEventListener('click', () => {
        universe.move_down();
    });
    document.getElementById('button-left').addEventListener('click', () => {
        universe.move_left();
    });
    document.getElementById('button-right').addEventListener('click', () => {
        universe.move_right();
    });
    document.getElementById('button-teleport').addEventListener('click', () => {
        universe.teleport();
    });
    document.getElementById('button-dynamite').addEventListener('click', () => {
        universe.place_dynamite();
    });

    window.addEventListener("devicemotion", (event) => {
        if (event.accelerationIncludingGravity.y > 5) {
            motion_request = 1;
            console.log("up");
        } else if (event.accelerationIncludingGravity.y < -5) {
            motion_request = 2;
            console.log("down");
        } else if (event.accelerationIncludingGravity.x < -5) {
            motion_request = 3;
            console.log("left");
        } else if (event.accelerationIncludingGravity.x > 5) {
            motion_request = 4;
            console.log("right");
        }
    }, true);

    document.addEventListener('keydown', (event) => {
        if ((event.key === "Up") || (event.key == "ArrowUp")) {
            universe.move_up();
        } else if ((event.key === "Down") || (event.key === "ArrowDown")) {
            universe.move_down();
        } else if ((event.key === "Left") || (event.key === "ArrowLeft")) {
            universe.move_left();
        } else if ((event.key === "Right") || (event.key === "ArrowRight")) {
            universe.move_right();
        }
    });

    universe.initialize();
    var oldTimestamp = 0;
    function renderFrame(timestamp) {
        if (timestamp - oldTimestamp > 200) {

            if (motion_request == 1) {  // up
                universe.move_up();
            } else if (motion_request == 2) {  // down
                universe.move_down();
            } else if (motion_request == 3) {  // left
                universe.move_left();
            } else if (motion_request == 4) {  // right
                universe.move_right();
            }
            motion_request = 0;

            universe.render_frame();
            oldTimestamp = timestamp;
        }
        requestAnimationFrame(renderFrame);
    }
    requestAnimationFrame(renderFrame);
  })
  .catch(console.error);

