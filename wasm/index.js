var keyUp;
const rust = import('./pkg')
  .then(m => {
    let universe = m.Universe.new();
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
    universe.initialize();
    var oldTimestamp = 0;
    function renderFrame(timestamp) {
        if (timestamp - oldTimestamp > 200) {
            universe.render_frame();
            oldTimestamp = timestamp;
        }
        requestAnimationFrame(renderFrame);
    }
    requestAnimationFrame(renderFrame);
  })
  .catch(console.error);

