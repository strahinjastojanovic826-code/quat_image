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
* **Lossless RLE Compression:** Compact binary file saving and loading using Run-Length Encoding (RLE) to minimize disk usage.
* **Advanced Computer Vision:** Built-in morphological operations such as dilation for quaternary and binary mask analysis.
* **Standard Image Quantization:** Automatic luminance conversion and mapping from standard RGB8 image buffers into 4-level quaternary format.

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
quat_image = 0.5.0 
```

Configure your library crate type for FFI compilation:

```toml
[lib]
name = "quat_image"
crate-type = ["cdylib", "staticlib"]
```

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

---

### 1. C Integration (`main.c`)

```c
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

// Opaque handle for QuatImage
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

    // Cleanup memory
    quat_image_free(handle);
    printf("C example executed successfully.\n");
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
    const char* quat_image_get_last_error_message();
    void quat_image_free_error_message(char* ptr);
}

class QuatImage {
private:
    QuatImageHandle* handle;

    void check_error() {
        const char* err_ptr = quat_image_get_last_error_message();
        if (err_ptr) {
            std::string msg(err_ptr);
            quat_image_free_error_message(const_cast<char*>(err_ptr));
            throw std::runtime_error(msg);
        }
    }

public:
    QuatImage(unsigned int w, unsigned int h) {
        handle = quat_image_new(w, h);
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

    void get_histogram(std::size_t out_counts[4]) const {
        if (quat_image_get_histogram(handle, out_counts) != 0) {
            throw std::runtime_error("Failed to retrieve histogram");
        }
    }

    void replace_color(uint8_t target, uint8_t replacement) {
        if (quat_image_replace_color(handle, target, replacement) != 0) {
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
        std::cout << "C++ example executed successfully." << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
    return 0;
}
```

---

### 3. C# Integration (`Program.cs`)

```csharp
using System;
using System.Runtime.InteropServices;

public class QuatImageWrapper : IDisposable {
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
    private static extern IntPtr quat_image_get_last_error_message();

    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    private static extern void quat_image_free_error_message(IntPtr ptr);

    public QuatImageWrapper(uint w, uint h) {
        handle = quat_image_new(w, h);
        if (handle == IntPtr.Zero) ThrowError();
    }

    public void SetPixel(uint x, uint y, byte val) {
        if (quat_image_set_pixel(handle, x, y, val) != 0) ThrowError();
    }

    public void Rotate90Cw() {
        if (quat_image_rotate_90_cw(handle) != 0) ThrowError();
    }

    public void ReplaceColor(byte target, byte replacement) {
        if (quat_image_replace_color(handle, target, replacement) != 0) ThrowError();
    }

    private void ThrowError() {
        IntPtr ptr = quat_image_get_last_error_message();
        string msg = ptr != IntPtr.Zero ? Marshal.PtrToStringAnsi(ptr) : "Unknown error";
        if (ptr != IntPtr.Zero) quat_image_free_error_message(ptr);
        throw new Exception(msg);
    }

    public void Dispose() {
        if (handle != IntPtr.Zero) {
            quat_image_free(handle);
            handle = IntPtr.Zero;
        }
    }
}

class Program {
    static void Main() {
        try {
            using (var img = new QuatImageWrapper(4, 4)) {
                img.SetPixel(0, 0, 1);
                img.SetPixel(1, 0, 2);

                img.Rotate90Cw();
                img.ReplaceColor(1, 3);

                Console.WriteLine("C# example executed successfully.");
            }
        } catch (Exception ex) {
            Console.WriteLine($"Error: {ex.Message}");
        }
    }
}
```

---

### 4. Python Integration (`main.py`)

```python
import ctypes

# Load the shared library (adjust the filename for Windows .dll or macOS .dylib if needed)
lib = ctypes.CDLL("./libquat_image.so")

# Define argument and return types for FFI functions
lib.quat_image_new.argtypes = [ctypes.c_uint, ctypes.c_uint]
lib.quat_image_new.restype = ctypes.c_void_p

lib.quat_image_free.argtypes = [ctypes.c_void_p]

lib.quat_image_set_pixel.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_uint, ctypes.c_ubyte]
lib.quat_image_set_pixel.restype = ctypes.c_int

lib.quat_image_get_pixel.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_uint, ctypes.POINTER(ctypes.c_ubyte)]
lib.quat_image_get_pixel.restype = ctypes.c_int

lib.quat_image_rotate_90_cw.argtypes = [ctypes.c_void_p]
lib.quat_image_rotate_90_cw.restype = ctypes.c_int

lib.quat_image_get_histogram.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t)]
lib.quat_image_get_histogram.restype = ctypes.c_int

lib.quat_image_replace_color.argtypes = [ctypes.c_void_p, ctypes.c_ubyte, ctypes.c_ubyte]
lib.quat_image_replace_color.restype = ctypes.c_int

lib.quat_image_get_last_error_message.restype = ctypes.c_char_p
lib.quat_image_free_error_message.argtypes = [ctypes.c_char_p]

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

    def get_pixel(self, x, y):
        val = ctypes.c_ubyte()
        if lib.quat_image_get_pixel(self.handle, x, y, ctypes.byref(val)) != 0:
            self._err()
        return val.value

    def rotate_90_cw(self):
        if lib.quat_image_rotate_90_cw(self.handle) != 0:
            self._err()

    def get_histogram(self):
        counts = (ctypes.c_size_t * 4)()
        if lib.quat_image_get_histogram(self.handle, counts) != 0:
            self._err()
        return list(counts)

    def replace_color(self, target, replacement):
        if lib.quat_image_replace_color(self.handle, target, replacement) != 0:
            self._err()

    def _err(self):
        ptr = lib.quat_image_get_last_error_message()
        msg = ptr.decode('utf-8') if ptr else "Unknown error"
        if ptr:
            lib.quat_image_free_error_message(ptr)
        raise RuntimeError(msg)

if __name__ == "__main__":
    try:
        # Create a 4x4 quat image
        img = QuatImage(4, 4)
        img.set_pixel(0, 0, 1)
        img.set_pixel(1, 0, 2)

        # Rotate 90 degrees clockwise
        img.rotate_90_cw()

        # Get and print histogram
        hist = img.get_histogram()
        print(f"Python Histogram -> 0: {hist[0]}, 1: {hist[1]}, 2: {hist[2]}, 3: {hist[3]}")

        # Replace color
        img.replace_color(1, 3)
        print("Python example executed successfully.")
        
    except Exception as e:
        print(f"Error: {e}")
```