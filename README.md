`quat_image` is a high-performance Rust library designed for memory-constrained image processing by packing 4 pixels (2 bits each, values ranging from 0 to 3) into a single `u8` byte. It features robust error handling, optimized in-place transformations, and production-ready FFI (Foreign Function Interface) safety for seamless C, C++, C#, and Python integration.

---

## Features

* **Extreme Memory Efficiency:** Packs four 2-bit pixels per byte, minimizing memory footprint.
* **FFI Panic Safety:** Built-in `std::panic::catch_unwind` protection prevents Rust panics from propagating into host C applications.
* **Thread-Local Error Management:** Safely propagates detailed error messages across language boundaries.
* **Optimized Internal Layout & Raw Access:** Supports fast pixel access, raw data pointers (`quat_image_data_ptr`), and zero-cost abstractions for high-performance loops.
* **Advanced Transformations & Analysis:** Includes 90° rotation, cropping, color replacement, and histogram generation.
* **Binary Persistence:** Built-in file saving and loading with custom magic header verification.
* **SIMD & Batch Optimization:** High-performance vectorized block packing and unpacking for maximum CPU throughput.
* **Multithreading Support:** Parallel pixel processing across multiple CPU cores using Rayon for heavy image workloads.
* **Advanced Computer Vision:** Built-in morphological operations such as dilation for quaternary and binary mask analysis.
* **Standard Image Quantization:** Automatic luminance conversion and mapping from standard RGB8 image buffers into 4-level quaternary format.
* **Multi-Channel Quaternary Images:** Supports 4-channel packed pixel layouts (`MultiQuatImage`) storing 2 bits per channel within a single byte.
* **Bitwise, Logic & LUT Operations:** `invert`, `invert_par`, 4-entry `remap` (with 256-entry byte lookup table), `threshold`, and element-wise `bitwise_and`, `bitwise_or`, and `bitwise_xor`.
* **Spatial & Connectivity Operations:** `blit`, `blit_masked`, `flood_fill`, and `connected_components`.
* **Conversions & Metrics:** `from_grayscale8`, `to_grayscale8`, `to_rgb8`, `diff_count`, `psnr`, and `iou`.
* **Cache-Friendly & Parallel Algorithms (`src/ops.rs`):** `rotate_90_cw_tiled` (16x16 cache tiles), `dilate_par`, `erode_par`, `opening_par`, `closing_par`, `histogram_par`, and `from_rgb8_par`.
* **SIMD Packing & Unpacking (`src/packing.rs`):** `pack_quats_simd` and `unpack_quats_simd` with runtime feature detection (`AVX2`, `SSSE3`, `NEON`) and scalar fallback.
* **Modular Feature Flags (`Cargo.toml`, `src/lib.rs`):** `default`, `io`, `ffi`, and `parallel`.
* **Zero-Copy Views (`src/view.rs`):** `QuatImageView<'a>` with `view`, `subview`, `get_pixel`, `histogram`, and `to_owned_image`.
* **Binary Format v1 & RLE Loading (`src/io.rs`):** `save_to_file_v1` and `load_from_file_v1` (`QIMG` v1 header with version and `CRC32` check), plus `load_from_file_rle`.
* **Test & Benchmark Suite (`tests/new_features_bench.rs`):** Integration tests covering all newly added features with per-test `Instant` timing output.

---

## Benchmark

| Benchmark / Test Name | Category | Measured Time |
| :--- | :--- | :--- |
| `test_01_basic_creation` | Image creation & dimensions | `10.3µs` |
| `test_02_set_and_get_pixel` | Pixel read/write access | `7.6µs` |
| `test_03_clear_image` | Buffer reset / clear | `1.5µs` |
| `test_04_pack_unpack_unit` | Quaternary bit packing/unpacking | `100ns` |
| `test_05_invalid_pixel_value` | Validation error (`> 3`) | `100ns` |
| `test_06_out_of_bounds` | Bounds check error | `100ns` |
| `test_07_invalid_dimensions` | Zero dimensions check | `0ns` |
| `test_08_flip_horizontal` | Horizontal image flip | `7.6µs` |
| `test_08_flip_vertical` | Vertical image flip | `2.2µs` |
| `test_09_integer_overflow_protection` | Overflow protection (`usize::MAX`) | `200ns` |
| `test_10_write_1M_pixels` | Stress write (`1000x1000` grid) | `8.3432ms` |
| `test_10_read_1M_pixels` | Stress read & verify (`1000x1000` grid) | `52.1171ms` |
| `test_10_flips_1M_pixels` | Stress flips (`1000x1000` grid) | `144.0331ms` |
| `test_15_quaternary_bitwise_and_invert` | Bitwise ops & invert | `15µs` |
| `test_16_remap_and_threshold` | LUT remap & thresholding | `38µs` |
| `test_17_blit_flood_fill_and_connected_components` | Blit, flood fill & components | `13.5µs` |
| `test_18_grayscale_rgb_and_metrics` | Conversions & quality metrics | `9.1µs` |
| `test_19_zero_copy_view_and_subview` | Zero-copy views & subviews | `3µs` |
| `test_20_simd_pack_unpack_roundtrip` | SIMD pack/unpack roundtrip | `16.8µs` |
| `test_21_tiled_rotate_and_parallel_ops` | Cache-tiled rotation & parallel ops | `1.6399ms` |
| `test_22_binary_v1_and_rle_roundtrip` | Binary v1 & RLE file roundtrip | `1.078ms` |
| `test_histogram` | Histogram calculation | `600ns` |
| `test_pack_quats_slice_batch` | Batch slice packing | `1µs` |
| `test_replace_color_parallel` | Parallel color replacement | `47.6µs` |
| `test_rgb8_quantization_conversion` | RGB8 quantization (`2x2`) | `1.9µs` |
| `test_rotate_90_cw` | 90° clockwise rotation | `1.5µs` |
| `Crop (50..150)` | Cropping benchmark | `1.1728ms` |
| `Erosion (10x10)` | Morphological erosion (`10x10`) | `53.2µs` |
| `Opening (100x100)` | Morphological opening (`100x100`) | `8.3432ms` |
| `Closing (100x100)` | Morphological closing (`100x100`) | `8.5396ms` |

---

## Licensing

This project is dual-licensed:

* **Open Source (AGPLv3):** Free to use for open-source projects. If you use or modify this library in your software or network service, your source code must also be open-source under AGPLv3.
* **Commercial License:** Required if you want to use `quat_image` in closed-source or proprietary commercial software without AGPLv3 restrictions.

For commercial licensing, contact: **strahinjastojanovic826@gmail.com**

---

## Installation

Add the library to your `Cargo.toml`:

```toml
[dependencies]
quat_image = "0.7.1"
```

Configure your library crate type for FFI compilation:

```toml
[lib]
name = "quat_image"
crate-type = ["cdylib", "staticlib"]
```

---

### Cargo Features 

* **Modular Cargo Features (`io`, `ffi`, `parallel`):** Enable only the capabilities you need (`io` for file persistence, `ffi` for C/C++/C#/Python bindings, `parallel` for multi-threaded `rayon` operations) or run with `--no-default-features` for core in-memory operations.

---

## Quick Start (Rust Usage)

```rust
use quat_image::image::QuatImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new 4x4 packed quat image
    let mut img = QuatImage::new(4, 4)?;
    
    // Set sample pixel values (valid range: 0 to 3)
    img.set_pixel(0, 0, 1)?;
    img.set_pixel(1, 0, 2)?;
    img.set_pixel(2, 0, 3)?;

    // Rotate the image 90 degrees clockwise
    img.rotate_90_cw()?;

    // Calculate histogram of quat values (0, 1, 2, 3)
    let hist = img.histogram();
    println!("Histogram -> 0: {}, 1: {}, 2: {}, 3: {}", hist[0], hist[1], hist[2], hist[3]);

    // Replace color values across the entire image
    img.replace_color(1, 3)?;

    // Crop a sub-region (x: 0, y: 0, width: 2, height: 2)
    let cropped = img.crop(0, 0, 2, 2)?;
    println!("Cropped image dimensions: {}x{}", cropped.width(), cropped.height());

    Ok(())
}
```

---

### More Examples (Views, Logic, Spatial Ops & Binary v1)

```rust
use quat_image::image::QuatImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = QuatImage::new(8, 8)?;
    img.set_pixel(1, 1, 2)?;
    img.set_pixel(2, 1, 2)?;
    img.set_pixel(1, 2, 2)?;
    img.set_pixel(2, 2, 2)?;

    // 1. Zero-copy view over a sub-region (no allocation or copy)
    let window = img.view(1, 1, 4, 4)?;
    let _hist_window = window.histogram();

    // 2. LUT remap & threshold
    img.remap([0, 1, 3, 3])?;
    img.threshold(2, 0, 3)?;

    // 3. Spatial flood fill & connected components
    let _filled = img.flood_fill(1, 1, 2)?;
    let (_labels, _comp_count) = img.connected_components();

    // 4. Quality metric (IoU) & binary v1 persistence with CRC32
    let _iou_score = img.iou(&img, 2)?;
    img.save_to_file_v1("mask_v1.qimg")?;
    let _loaded = QuatImage::load_from_file_v1("mask_v1.qimg")?;

    Ok(())
}
```

---

## Advanced Usage 

You can leverage multi-threading, computer vision operations, standard RGB conversion, and lossless RLE compression:

```rust
use quat_image::image::QuatImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Convert standard RGB8 image buffer into quaternary format (Quantization)
    let rgb_data = vec![
        10, 10, 10,     // Maps to quat 0
        90, 90, 90,     // Maps to quat 1
        160, 160, 160,  // Maps to quat 2
        250, 250, 250,  // Maps to quat 3
    ];
    let mut img = QuatImage::from_rgb8(2, 2, &rgb_data)?;

    // 2. Parallel color replacement across multiple CPU cores (Rayon multithreading)
    img.replace_color_par(1, 3)?;

    // 3. Apply morphological dilation (Computer Vision 3x3 max filter)
    let dilated = img.dilate()?;

    // 4. Save image using compact lossless RLE compression
    dilated.save_to_file_rle("compressed_output.qimg")?;

    Ok(())
}
```

---

## FFI Integration & Multi-Language Usage

The library exposes a C-compatible API (`extern "C"`) allowing seamless interaction from other programming languages.

### Available FFI Functions:
* `quat_image_new(width, height)` - Allocates a new image handle.
* `quat_image_free(handle)` - Frees the image memory.
* `quat_image_width(handle)` / `quat_image_height(handle)` - Retrieves dimensions.
* `quat_image_get_pixel(handle, x, y, result)` - Gets a pixel value safely.
* `quat_image_set_pixel(handle, x, y, val)` - Sets a pixel value safely.
* `quat_image_data_ptr(handle)` / `quat_image_data_mut_ptr(handle)` - Direct raw buffer access.
* `quat_image_get_last_error_message()` - Retrieves error details if an FFI call fails.
* `quat_image_free_error_message(ptr)` - Cleans up error string memory.
* `quat_image_replace_color_par(handle, target, replacement)` - Replaces target color in parallel across CPU cores.
* `quat_image_erode(handle)` - Applies 3x3 morphological erosion, returning a new handle.
* `quat_image_dilate(handle)` - Applies 3x3 morphological dilation, returning a new handle.
* `quat_image_data_len(handle)` - Retrieves the length of the packed byte buffer.
* `quat_image_copy_bytes(handle, dest, dest_cap)` - Safely copies raw packed bytes into a destination buffer.

---

### 1. C Integration (`main.c`)

```c
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

typedef struct QuatImageHandle QuatImageHandle;

// External FFI functions
extern QuatImageHandle* quat_image_new(unsigned int width, unsigned int height);
extern void quat_image_free(QuatImageHandle* handle);
extern int quat_image_width(const QuatImageHandle* handle);
extern int quat_image_height(const QuatImageHandle* handle);
extern int quat_image_get_pixel(const QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t* result);
extern int quat_image_set_pixel(QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t val);
extern int quat_image_rotate_90_cw(QuatImageHandle* handle);
extern int quat_image_get_histogram(const QuatImageHandle* handle, size_t* out_counts);
extern int quat_image_replace_color(QuatImageHandle* handle, uint8_t target, uint8_t replacement);
extern QuatImageHandle* quat_image_erode(QuatImageHandle* handle);
extern QuatImageHandle* quat_image_dilate(QuatImageHandle* handle);
extern int quat_image_replace_color_par(QuatImageHandle* handle, uint8_t target, uint8_t replacement);
extern const char* quat_image_get_last_error_message(void);
extern void quat_image_free_error_message(char* ptr);

int main() {
    // Create a new 4x4 image
    QuatImageHandle* handle = quat_image_new(4, 4);
    if (!handle) {
        const char* err = quat_image_get_last_error_message();
        fprintf(stderr, "Failed to create image: %s\n", err ? err : "Unknown");
        if (err) quat_image_free_error_message((char*)err);
        return 1;
    }

    // Set pixels
    quat_image_set_pixel(handle, 0, 0, 1);
    quat_image_set_pixel(handle, 1, 0, 2);

    // Rotate 90 degrees clockwise
    quat_image_rotate_90_cw(handle);

    // Get histogram
    size_t histogram[4];
    if (quat_image_get_histogram(handle, histogram) == 0) {
        printf("Histogram -> 0: %zu, 1: %zu, 2: %zu, 3: %zu\n", 
               histogram[0], histogram[1], histogram[2], histogram[3]);
    }

    // Replace color
    quat_image_replace_color(handle, 1, 3);

    // Apply morphological dilation
    QuatImageHandle* dilated = quat_image_dilate(handle);
    if (dilated) {
        printf("Dilation executed successfully.\n");
        quat_image_free(dilated);
    }

    // Parallel color replacement across multi-cores
    quat_image_replace_color_par(handle, 2, 3);

    // Cleanup memory
    quat_image_free(handle);
    printf("C example executed successfully.\n");
    return 0;
}
```

---

### Additional FFI Examples (C, C++ & C#)

#### C (`extra_ffi.c`)
```c
#include <stdint.h>
#include <stdio.h>

typedef struct QuatImageHandle QuatImageHandle;
extern QuatImageHandle* quat_image_new(unsigned int width, unsigned int height);
extern void quat_image_free(QuatImageHandle* handle);
extern int quat_image_set_pixel(QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t val);
extern int quat_image_get_pixel(const QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t* result);

int run_extra_c(void) {
    QuatImageHandle* img = quat_image_new(8, 8);
    if (!img) return 1;

    quat_image_set_pixel(img, 2, 3, 2);
    uint8_t px = 0;
    if (quat_image_get_pixel(img, 2, 3, &px) == 0) {
        printf("Pixel at (2, 3): %u\n", px);
    }

    quat_image_free(img);
    return 0;
}
```

---

### 2. C++ Integration (`main.cpp`)

```cpp
#include <iostream>
#include <cstdint>
#include <stdexcept>
#include <string>

extern "C" {
    struct QuatImageHandle;
    QuatImageHandle* quat_image_new(unsigned int width, unsigned int height);
    void quat_image_free(QuatImageHandle* handle);
    int quat_image_width(const QuatImageHandle* handle);
    int quat_image_height(const QuatImageHandle* handle);
    int quat_image_get_pixel(const QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t* result);
    int quat_image_set_pixel(QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t val);
    int quat_image_rotate_90_cw(QuatImageHandle* handle);
    int quat_image_get_histogram(const QuatImageHandle* handle, std::size_t* out_counts);
    int quat_image_replace_color(QuatImageHandle* handle, uint8_t target, uint8_t replacement);
    QuatImageHandle* quat_image_erode(QuatImageHandle* handle);
    QuatImageHandle* quat_image_dilate(QuatImageHandle* handle);
    int quat_image_replace_color_par(QuatImageHandle* handle, uint8_t target, uint8_t replacement);
    const char* quat_image_get_last_error_message(void);
    void quat_image_free_error_message(char* ptr);
}

class QuatImage {
private:
    QuatImageHandle* handle;

    void check_error() const {
        const char* err_ptr = quat_image_get_last_error_message();
        if (err_ptr) {
            std::string msg = err_ptr;
            quat_image_free_error_message(const_cast<char*>(err_ptr));
            throw std::runtime_error(msg);
        }
    }

public:
    QuatImage(unsigned int w, unsigned int h) {
        handle = quat_image_new(w, h);
        if (!handle) check_error();
    }

    explicit QuatImage(QuatImageHandle* h) : handle(h) {
        if (!handle) check_error();
    }

    ~QuatImage() {
        if (handle) {
            quat_image_free(handle);
        }
    }

    void set_pixel(unsigned int x, unsigned int y, uint8_t val) {
        if (quat_image_set_pixel(handle, x, y, val) != 0) {
            check_error();
        }
    }

    void rotate_90_cw() {
        if (quat_image_rotate_90_cw(handle) != 0) {
            check_error();
        }
    }

    std::size_t* get_histogram(std::size_t out_counts[4]) const {
        if (quat_image_get_histogram(handle, out_counts) != 0) {
            throw std::runtime_error("Failed to retrieve histogram");
        }
        return out_counts;
    }

    void replace_color(uint8_t target, uint8_t replacement) {
        if (quat_image_replace_color(handle, target, replacement) != 0) {
            check_error();
        }
    }

    QuatImage erode() const {
        QuatImageHandle* res_handle = quat_image_erode(handle);
        if (!res_handle) check_error();
        return QuatImage(res_handle);
    }

    QuatImage dilate() const {
        QuatImageHandle* res_handle = quat_image_dilate(handle);
        if (!res_handle) check_error();
        return QuatImage(res_handle);
    }

    void replace_color_par(uint8_t target, uint8_t replacement) {
        if (quat_image_replace_color_par(handle, target, replacement) != 0) {
            check_error();
        }
    }
};

int main() {
    try {
        QuatImage img(4, 4);
        img.set_pixel(0, 0, 1);
        img.set_pixel(1, 0, 2);

        img.rotate_90_cw();

        std::size_t hist[4];
        img.get_histogram(hist);
        std::cout << "C++ Histogram -> 0: " << hist[0] << ", 1: " << hist[1] 
                  << ", 2: " << hist[2] << ", 3: " << hist[3] << std::endl;

        img.replace_color(1, 3);

        QuatImage dilated = img.dilate();
        img.replace_color_par(2, 3);

        std::cout << "C++ example executed successfully." << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
    return 0;
}
```

---

#### C++ (`extra_ffi.cpp`)
```cpp
#include <cstdint>
#include <iostream>

extern "C" {
struct QuatImageHandle;
QuatImageHandle* quat_image_new(unsigned int width, unsigned int height);
void quat_image_free(QuatImageHandle* handle);
int quat_image_set_pixel(QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t val);
int quat_image_get_pixel(const QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t* result);
}

void run_extra_cpp() {
    QuatImageHandle* raw = quat_image_new(4, 4);
    if (!raw) return;

    quat_image_set_pixel(raw, 1, 1, 3);
    std::uint8_t val = 0;
    if (quat_image_get_pixel(raw, 1, 1, &val) == 0) {
        std::cout << "C++ pixel (1, 1) = " << static_cast<int>(val) << "\n";
    }
    quat_image_free(raw);
}
```

---

### 3. C# Integration (`Program.cs`)

```csharp
using System;
using System.Runtime.InteropServices;

namespace QuatImageDemo
{
    public class QuatImageWrapper : IDisposable
    {
        private IntPtr handle;

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr quat_image_new(uint width, uint height);

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern void quat_image_free(IntPtr handle);

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern int quat_image_set_pixel(IntPtr handle, uint x, uint y, byte val);

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern int quat_image_rotate_90_cw(IntPtr handle);

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern int quat_image_replace_color(IntPtr handle, byte target, byte replacement);

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr quat_image_erode(IntPtr handle);

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr quat_image_dilate(IntPtr handle);

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern int quat_image_replace_color_par(IntPtr handle, byte target, byte replacement);

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr quat_image_get_last_error_message();

        [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
        private static extern void quat_image_free_error_message(IntPtr ptr);

        public QuatImageWrapper(uint w, uint h)
        {
            handle = quat_image_new(w, h);
            if (handle == IntPtr.Zero) ThrowError();
        }

        private QuatImageWrapper(IntPtr existingHandle)
        {
            handle = existingHandle;
            if (handle == IntPtr.Zero) ThrowError();
        }

        public void SetPixel(uint x, uint y, byte val)
        {
            if (quat_image_set_pixel(handle, x, y, val) != 0) ThrowError();
        }

        public void Rotate90Cw()
        {
            if (quat_image_rotate_90_cw(handle) != 0) ThrowError();
        }

        public void ReplaceColor(byte target, byte replacement)
        {
            if (quat_image_replace_color(handle, target, replacement) != 0) ThrowError();
        }

        public QuatImageWrapper Dilate()
        {
            IntPtr resHandle = quat_image_dilate(handle);
            if (resHandle == IntPtr.Zero) ThrowError();
            return new QuatImageWrapper(resHandle);
        }

        public void ReplaceColorPar(byte target, byte replacement)
        {
            if (quat_image_replace_color_par(handle, target, replacement) != 0) ThrowError();
        }

        private void ThrowError()
        {
            IntPtr ptr = quat_image_get_last_error_message();
            string msg = ptr != IntPtr.Zero ? Marshal.PtrToStringAnsi(ptr) : "Unknown error";
            if (ptr != IntPtr.Zero) quat_image_free_error_message(ptr);
            throw new Exception(msg);
        }

        public void Dispose()
        {
            if (handle != IntPtr.Zero)
            {
                quat_image_free(handle);
                handle = IntPtr.Zero;
            }
        }
    }

    class Program
    {
        static void Main(string[] args)
        {
            try
            {
                using (var img = new QuatImageWrapper(4, 4))
                {
                    img.SetPixel(0, 0, 1);
                    img.SetPixel(1, 0, 2);

                    img.Rotate90Cw();
                    img.ReplaceColor(1, 3);

                    using (var dilated = img.Dilate())
                    {
                        img.ReplaceColorPar(2, 3);
                    }

                    Console.WriteLine("C# example executed successfully.");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error: {ex.Message}");
            }
        }
    }
}
```

---

#### C# (`ExtraFfi.cs`)
```csharp
using System;
using System.Runtime.InteropServices;

static class ExtraQuatInterop
{
    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr quat_image_new(uint width, uint height);

    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    public static extern void quat_image_free(IntPtr handle);

    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    public static extern int quat_image_set_pixel(IntPtr handle, uint x, uint y, byte val);

    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    public static extern int quat_image_get_pixel(IntPtr handle, uint x, uint y, out byte result);
}
```

---

### 4. Python Integration (`main.py`)

```python
import ctypes

# Load the shared library
lib = ctypes.CDLL("./libquat_image.so")  # or .dll on Windows

# Define FFI signatures
lib.quat_image_new.argtypes = [ctypes.c_uint, ctypes.c_uint]
lib.quat_image_new.restype = ctypes.c_void_p

lib.quat_image_free.argtypes = [ctypes.c_void_p]
lib.quat_image_free.restype = None

lib.quat_image_set_pixel.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_uint, ctypes.c_ubyte]
lib.quat_image_set_pixel.restype = ctypes.c_int

lib.quat_image_rotate_90_cw.argtypes = [ctypes.c_void_p]
lib.quat_image_rotate_90_cw.restype = ctypes.c_int

lib.quat_image_replace_color.argtypes = [ctypes.c_void_p, ctypes.c_ubyte, ctypes.c_ubyte]
lib.quat_image_replace_color.restype = ctypes.c_int

lib.quat_image_erode.argtypes = [ctypes.c_void_p]
lib.quat_image_erode.restype = ctypes.c_void_p

lib.quat_image_dilate.argtypes = [ctypes.c_void_p]
lib.quat_image_dilate.restype = ctypes.c_void_p

lib.quat_image_replace_color_par.argtypes = [ctypes.c_void_p, ctypes.c_ubyte, ctypes.c_ubyte]
lib.quat_image_replace_color_par.restype = ctypes.c_int

lib.quat_image_get_last_error_message.restype = ctypes.c_char_p
lib.quat_image_free_error_message.argtypes = [ctypes.c_void_p]


class QuatImage:
    def __init__(self, w, h):
        self.handle = lib.quat_image_new(w, h)
        if not self.handle:
            self._err()

    def __del__(self):
        if hasattr(self, 'handle') and self.handle:
            lib.quat_image_free(self.handle)

    def set_pixel(self, x, y, v):
        if lib.quat_image_set_pixel(self.handle, x, y, v) != 0:
            self._err()

    def rotate_90_cw(self):
        if lib.quat_image_rotate_90_cw(self.handle) != 0:
            self._err()

    def replace_color(self, target, replacement):
        if lib.quat_image_replace_color(self.handle, target, replacement) != 0:
            self._err()

    def erode(self):
        res_handle = lib.quat_image_erode(self.handle)
        if not res_handle:
            self._err()
        new_img = QuatImage.__new__(QuatImage)
        new_img.handle = res_handle
        return new_img

    def dilate(self):
        res_handle = lib.quat_image_dilate(self.handle)
        if not res_handle:
            self._err()
        new_img = QuatImage.__new__(QuatImage)
        new_img.handle = res_handle
        return new_img

    def replace_color_par(self, target, replacement):
        if lib.quat_image_replace_color_par(self.handle, target, replacement) != 0:
            self._err()

    def _err(self):
        ptr = lib.quat_image_get_last_error_message()
        msg = ptr.decode('utf-8') if ptr else "Unknown error"
        if ptr:
            lib.quat_image_free_error_message(ptr)
        raise RuntimeError(msg)


if __name__ == "__main__":
    try:
        img = QuatImage(4, 4)
        img.set_pixel(0, 0, 1)
        img.set_pixel(1, 0, 2)

        img.rotate_90_cw()
        img.replace_color(1, 3)

        dilated = img.dilate()
        img.replace_color_par(2, 3)

        print("Python example executed successfully.")
    except Exception as e:
        print(f"Error: {e}")
```

---

### Additional Python Example (`extra_main.py`)

```python
import ctypes


def run_extra_python(lib: ctypes.CDLL) -> None:
    lib.quat_image_new.argtypes = [ctypes.c_uint, ctypes.c_uint]
    lib.quat_image_new.restype = ctypes.c_void_p
    lib.quat_image_free.argtypes = [ctypes.c_void_p]
    lib.quat_image_free.restype = None

    lib.quat_image_set_pixel.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint,
        ctypes.c_uint,
        ctypes.c_ubyte,
    ]
    lib.quat_image_set_pixel.restype = ctypes.c_int

    lib.quat_image_get_pixel.argtypes = [
        ctypes.c_void_p,
        ctypes.c_uint,
        ctypes.c_uint,
        ctypes.POINTER(ctypes.c_ubyte),
    ]
    lib.quat_image_get_pixel.restype = ctypes.c_int

    handle = lib.quat_image_new(8, 8)
    if not handle:
        raise RuntimeError("Failed to allocate QuatImage")

    try:
        if lib.quat_image_set_pixel(handle, 3, 4, 2) != 0:
            raise RuntimeError("Failed to set pixel")

        pixel = ctypes.c_ubyte(0)
        if lib.quat_image_get_pixel(handle, 3, 4, ctypes.byref(pixel)) != 0:
            raise RuntimeError("Failed to get pixel")

        print(f"Python extra pixel (3, 4) = {pixel.value}")
    finally:
        lib.quat_image_free(handle)
```