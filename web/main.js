"use strict";

function string_from_rust(instance, address, length) {
    const octets = new Uint8Array(instance.exports.memory.buffer, address, length);
    return new TextDecoder().decode(octets);
}

function string_to_rust(instance, s) {
    const octets = new TextEncoder("utf-8").encode(s);
    const address = instance.exports.alloc(octets.length);

    let view = new Uint8Array(instance.exports.memory.buffer);
    view.set(octets, address);

    instance.exports.stack_push(address);
    instance.exports.stack_push(octets.length);
}

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
    const init = () => {
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
    };

    const tick = timestamp => {
        if (game_object_address === null) {
            init();
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
        const cx = x * BLOCK_SIZE_PX + 1;
        const cy = y * BLOCK_SIZE_PX + 1;
        const cw = BLOCK_SIZE_PX - 2;
        const ch = BLOCK_SIZE_PX - 2;

        if (color == 0) {
            ctx.globalCompositeOperation = "source-over";
            ctx.fillStyle = "#000000";
            ctx.fillRect(cx, cy, cw, ch);
        } else {
            ctx.globalCompositeOperation = "source-over";
            ctx.drawImage(tile_image, cx, cy, cw, ch);

            ctx.globalCompositeOperation = "multiply";
            ctx.fillStyle = "#" + color.toString(16).padStart(6, "0");
            ctx.fillRect(cx, cy, cw, ch);
        }
    };

    const console_log = (address, length) => {
        const s = string_from_rust(wasm_instance, address, length);
        console.log(s);
    };

    const random = () => {
        return Math.random();
    };

    const html = (id_addr, id_len, html_addr, html_len) => {
        const id = string_from_rust(wasm_instance, id_addr, id_len);
        const html = string_from_rust(wasm_instance, html_addr, html_len);
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
    const shim_handler = {
        get: function (obj, prop) {
            return function () { return shims[prop].apply(this, arguments); };
        }
    };

    const importObject = {
        env: new Proxy({}, shim_handler),
    };

    fetch("rstetris.wasm")
        .then(response => response.arrayBuffer())
        .then(result => WebAssembly.instantiate(result, importObject))
        .then(result => {
            shims = register_object(result.module, result.instance);
        })
        .catch(error => console.log("Error loading WASM module: " + error));
})();
