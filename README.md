# wasm-pngquant

A WebAssembly (Wasm) port of the industry-standard `pngquant` lossy PNG compressor. 

This library allows you to shrink PNG images by up to 70% **directly in the user's web browser**, with no server uploads required. It leverages the Rust implementation of `libimagequant` and outputs fully formatted, indexed 8-bit PNG files ready for display or download.

## Features
- 🚀 **Fast:** Runs at near-native speeds using WebAssembly.
- 🔒 **Private:** Compression happens client-side; no images are sent to a server.
- 📦 **Drop-in Ready:** Generates standard `.png` files via the `png` Rust crate.

## How to use

1. Download the latest release from the [Releases tab](../../releases) and extract the `pkg/` folder into your web project.
2. Load the module and pass it raw RGBA pixel data from an HTML Canvas.

```html
<input type="file" id="upload" accept="image/png, image/jpeg" />
<img id="result-image" />

<script type="module">
    import init, { compress_png_data } from './pkg/wasm_pngquant.js';

    async function start() {
        await init(); // Boot WebAssembly
        
        document.getElementById('upload').addEventListener('change', async (event) => {
            const file = event.target.files[0];
            
            // 1. Load image and draw to an off-screen canvas to get raw pixels
            const img = new Image();
            img.src = URL.createObjectURL(file);
            await new Promise(r => img.onload = r);
            
            const canvas = document.createElement('canvas');
            canvas.width = img.width;
            canvas.height = img.height;
            const ctx = canvas.getContext('2d');
            ctx.drawImage(img, 0, 0);
            
            const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

            // 2. Compress! (Returns a Uint8Array containing a new .png file)
            const compressedBytes = compress_png_data(
                imageData.data, 
                canvas.width, 
                canvas.height
            );

            // 3. Display the result
            const blob = new Blob([compressedBytes], { type: 'image/png' });
            document.getElementById('result-image').src = URL.createObjectURL(blob);
        });
    }
    
    start();
</script>
```

> **Note:** Due to browser security policies (CORS), WebAssembly cannot be loaded via `file://`. You must serve your files using a local web server (e.g., `python3 -m http.server 8000`).

## Local Development

To compile this project from source, you will need [Rust](https://rustup.rs/) and `wasm-pack`.

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build the WebAssembly package
wasm-pack build --target web
```

This will generate the `pkg/` directory containing the compiled `.wasm` binary and JavaScript bindings.

## License

This project uses `libimagequant`, which is licensed under the **GPLv3**. Therefore, this project is also licensed under the GPLv3. If you wish to use this in a closed-source commercial product, you must obtain a commercial license from the imagequant developers.