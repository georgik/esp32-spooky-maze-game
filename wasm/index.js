var keyUp;

const motionDirection = {
    none: 0,
    up: 1,
    down: 2,
    left: 3,
    right: 4
};

const tiltThreshold = 2;

const rust = import('./pkg')
  .then(m => {
    let universe = m.Universe.new();
    var motionRequestHorizontal = motionDirection.none;
    var motionRequestVertical = motionDirection.none;

    document.getElementById('html-console').innerHTML = "Initialize";

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
        
        if (event.accelerationIncludingGravity.y > tiltThreshold) {
            motionRequestVertical = motionDirection.down;
        } else if (event.accelerationIncludingGravity.y < -tiltThreshold) {
            motionRequestVertical = motionDirection.up;
        }
        
        if (event.accelerationIncludingGravity.x < -tiltThreshold) {
            motionRequestHorizontal = motionDirection.right;
        } else if (event.accelerationIncludingGravity.x > tiltThreshold) {
            motionRequestHorizontal = motionDirection.left;
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

            if (motionRequestVertical == motionDirection.up) {  // up
                universe.move_up();
            } else if (motionRequestVertical == motionDirection.down) {  // down
                universe.move_down();
            }
            
            if (motionRequestHorizontal == motionDirection.left) {  // left
                universe.move_left();
            } else if (motionRequestHorizontal == motionDirection.right) {  // right
                universe.move_right();
            }
            motionRequestVertical = motionDirection.none;
            motionRequestHorizontal = motionDirection.none;

            universe.render_frame();
            oldTimestamp = timestamp;
        }
        requestAnimationFrame(renderFrame);
    }
    requestAnimationFrame(renderFrame);
    document.getElementById('html-console').innerHTML = "Game is running";
  })
  .catch(console.error);

