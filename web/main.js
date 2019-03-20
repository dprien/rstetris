"use strict";

function register_object(wasm_module, wasm_instance) {
    const BOARD_WIDTH = 10;
    const BOARD_HEIGHT = 20;

    const BLOCK_SIZE_PX = 50;

    const BOARD_WIDTH_PX = BLOCK_SIZE_PX * BOARD_WIDTH;
    const BOARD_HEIGHT_PX = BLOCK_SIZE_PX * BOARD_HEIGHT;

    const canvas = document.getElementById("the_canvas");
    canvas.width = BOARD_WIDTH_PX;
    canvas.height = BOARD_HEIGHT_PX;
    const ctx = canvas.getContext("2d");

    let tile_image = new Image();
    tile_image.src = "tile.png";

    let game_object_address = null;
    const tick = timestamp => {
        if (game_object_address === null) {
            ctx.clearRect(0, 0, BOARD_WIDTH_PX, BOARD_HEIGHT_PX);
            draw_grid();

            game_object_address = wasm_instance.exports.Game_new(BOARD_WIDTH, BOARD_HEIGHT);

            window.addEventListener("keydown", e => { wasm_instance.exports.Game_key_handler(game_object_address, e.keyCode, 1); }, false);
            window.addEventListener("keyup", e => { wasm_instance.exports.Game_key_handler(game_object_address, e.keyCode, 0); }, false);

            function call_touch_handlers(e, handler) {
                e.preventDefault();
                for (let i = 0; i < e.changedTouches.length; i++) {
                    let touch = e.changedTouches[i];
                    handler(game_object_address, touch.identifier, touch.pageX, touch.pageY);
                }
            }

            canvas.addEventListener("touchstart", e => { call_touch_handlers(e, wasm_instance.exports.Game_touch_start_handler); }, false);
            canvas.addEventListener("touchend", e => { call_touch_handlers(e, wasm_instance.exports.Game_touch_end_handler); }, false);
            canvas.addEventListener("touchcancel", e => { call_touch_handlers(e, wasm_instance.exports.Game_touch_cancel_handler); }, false);
            canvas.addEventListener("touchmove", e => { call_touch_handlers(e, wasm_instance.exports.Game_touch_move_handler); }, false);
        }

        wasm_instance.exports.Game_tick(game_object_address, timestamp);
        requestAnimationFrame(tick);
    };

    const draw_grid = () => {
        ctx.beginPath();

        ctx.strokeStyle = "#001717";
        ctx.lineWidth = 2.0;

        for (let x = 1; x < BOARD_WIDTH; x++) {
            ctx.moveTo(x * BLOCK_SIZE_PX, 0);
            ctx.lineTo(x * BLOCK_SIZE_PX, BOARD_HEIGHT_PX - 1);
        }

        for (let y = 1; y < BOARD_HEIGHT; y++) {
            ctx.moveTo(0, y * BLOCK_SIZE_PX);
            ctx.lineTo(BOARD_WIDTH_PX - 1, y * BLOCK_SIZE_PX);
        }

        ctx.stroke();
    };

    const draw_block = (x, y, color) => {
        const color_hex = "#" + color.toString(16).padStart(6, "0");

        const cx = x * BLOCK_SIZE_PX + 1;
        const cy = y * BLOCK_SIZE_PX + 1;
        const cw = BLOCK_SIZE_PX - 2;
        const ch = BLOCK_SIZE_PX - 2;

        ctx.globalCompositeOperation = "source-over";
        ctx.drawImage(tile_image, cx, cy, cw, ch);

        ctx.globalCompositeOperation = "multiply";
        ctx.fillStyle = color_hex;
        ctx.fillRect(cx, cy, cw, ch);
    };

    const console_log = (address, length) => {
        const buffer = new Uint8Array(wasm_instance.exports.memory.buffer, address, length);
        const s = new TextDecoder().decode(buffer);
        console.log(s);
    };

    const random = () => {
        return Math.random();
    };

    const html = (id_addr, id_len, html_addr, html_len) => {
        const id_buf = new Uint8Array(wasm_instance.exports.memory.buffer, id_addr, id_len);
        const id = new TextDecoder().decode(id_buf);

        const html_buf = new Uint8Array(wasm_instance.exports.memory.buffer, html_addr, html_len);
        const html = new TextDecoder().decode(html_buf);

        document.getElementById(id).innerHTML = html;
    }

    requestAnimationFrame(tick);

    return {
        console_log: console_log,
        draw_block: draw_block,
        random: random,
        html: html,
    };
}

(function () {
    let shims = null;
    const importObject = {
        env: {
            console_log: function () { return shims.console_log.apply(this, arguments); },
            draw_block: function () { return shims.draw_block.apply(this, arguments); },
            random: function () { return shims.random.apply(this, arguments); },
            html: function () { return shims.html.apply(this, arguments); },
        }
    };

    fetch("rstetris.wasm")
        .then(response => response.arrayBuffer())
        .then(result => WebAssembly.instantiate(result, importObject))
        .then(result => {
            shims = register_object(result.module, result.instance);
        })
        .catch(error => console.log("Error loading WASM module: " + error));
})();
