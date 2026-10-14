`quat_image` is a high-performance Rust library designed for memory-constrained image processing by packing 4 pixels (2 bits each, values ranging from 0 to 3) into a single `u8` byte. It features robust error handling, optimized in-place transformations, and production-ready FFI (Foreign Function Interface) safety for seamless C, C++, C#, and Python integration.

---

## Features

* **Extreme Memory Efficiency:** Packs four 2-bit pixels per byte, reducing memory footprints by 75% compared to standard 8-bit grayscale images.
* **FFI Panic Safety:** Built-in `std::panic::catch_unwind` protection prevents Rust panics from crossing the FFI boundary and causing undefined behavior in host applications.
* **Thread-Local Error Management:** Safely propagates detailed error messages across language boundaries via thread-local storage.
* **Optimized Internal Layout & Raw Access:** Supports fast pixel access, in-place horizontal/vertical flips, and raw buffer pointers (`quat_image_data_ptr`) for high-performance loops.

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
quat_image = 0.3.0 
(happends sometime)
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
    // Create a new 10x10 packed image
    let mut img = QuatImage::new(10, 10)?;

    // Set a pixel at coordinate (2, 3) with value 3 (max 2-bit value)
    img.set_pixel(2, 3, 3)?;

    // Retrieve and verify the pixel value
    let val = img.get_pixel(2, 3)?;
    println!("Pixel value: {}", val); // Outputs: 3

    // Perform an in-place horizontal flip
    img.flip_horizontal()?;

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

typedef struct QuatImageHandle QuatImageHandle;

extern QuatImageHandle* quat_image_new(unsigned int width, unsigned int height);
extern void quat_image_free(QuatImageHandle* handle);
extern int quat_image_width(const QuatImageHandle* handle);
extern int quat_image_height(const QuatImageHandle* handle);
extern int quat_image_get_pixel(const QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t* result);
extern int quat_image_set_pixel(QuatImageHandle* handle, unsigned int x, unsigned int y, uint8_t val);
extern const char* quat_image_get_last_error_message(void);
extern void quat_image_free_error_message(char* ptr);

int main() {
    QuatImageHandle* handle = quat_image_new(10, 10);
    if (!handle) {
        const char* err = quat_image_get_last_error_message();
        fprintf(stderr, "Error: %s\n", err ? err : "Unknown");
        if (err) quat_image_free_error_message((char*)err);
        return 1;
    }

    quat_image_set_pixel(handle, 2, 3, 3);
    uint8_t val = 0;
    quat_image_get_pixel(handle, 2, 3, &val);
    printf("C Pixel: %u\n", val);

    quat_image_free(handle);
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
        if (!handle) { check_error(); throw std::runtime_error("Failed to create image"); }
    }
    ~QuatImage() { if (handle) quat_image_free(handle); }
    void set_pixel(unsigned int x, unsigned int y, uint8_t val) {
        if (quat_image_set_pixel(handle, x, y, val) != 0) check_error();
    }
    uint8_t get_pixel(unsigned int x, unsigned int y) const {
        uint8_t val = 0;
        if (quat_image_get_pixel(handle, x, y, &val) != 0) {
            const char* err_ptr = quat_image_get_last_error_message();
            std::string msg(err_ptr ? err_ptr : "Unknown error");
            if (err_ptr) quat_image_free_error_message(const_cast<char*>(err_ptr));
            throw std::runtime_error(msg);
        }
        return val;
    }
};

int main() {
    try {
        QuatImage img(10, 10);
        img.set_pixel(2, 3, 3);
        std::cout << "C++ Pixel: " << (int)img.get_pixel(2, 3) << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
    }
}
```

---

### 3. C# Integration (`Program.cs`)

```csharp
using System;
using System.Runtime.InteropServices;

public class QuatImage : IDisposable {
    IntPtr handle;
    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    static extern IntPtr quat_image_new(uint w, uint h);
    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    static extern void quat_image_free(IntPtr h);
    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    static extern int quat_image_set_pixel(IntPtr h, uint x, uint y, byte val);
    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    static extern int quat_image_get_pixel(IntPtr h, uint x, uint y, out byte res);
    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    static extern IntPtr quat_image_get_last_error_message();
    [DllImport("quat_image", CallingConvention = CallingConvention.Cdecl)]
    static extern void quat_image_free_error_message(IntPtr ptr);

    public QuatImage(uint w, uint h) {
        handle = quat_image_new(w, h);
        if (handle == IntPtr.Zero) ThrowError();
    }
    public void SetPixel(uint x, uint y, byte val) {
        if (quat_image_set_pixel(handle, x, y, val) != 0) ThrowError();
    }
    public byte GetPixel(uint x, uint y) {
        if (quat_image_get_pixel(handle, x, y, out byte val) != 0) ThrowError();
        return val;
    }
    void ThrowError() {
        IntPtr ptr = quat_image_get_last_error_message();
        string msg = ptr != IntPtr.Zero ? Marshal.PtrToStringAnsi(ptr) : "Error";
        if (ptr != IntPtr.Zero) quat_image_free_error_message(ptr);
        throw new Exception(msg);
    }
    public void Dispose() { if (handle != IntPtr.Zero) { quat_image_free(handle); handle = IntPtr.Zero; } }
}
```

---

### 4. Python Integration (`main.py`)

```python
import ctypes

lib = ctypes.CDLL("./libquat_image.so")
lib.quat_image_new.argtypes = [ctypes.c_uint, ctypes.c_uint]
lib.quat_image_new.restype = ctypes.c_void_p
lib.quat_image_free.argtypes = [ctypes.c_void_p]
lib.quat_image_set_pixel.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_uint, ctypes.c_ubyte]
lib.quat_image_get_pixel.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_uint, ctypes.POINTER(ctypes.c_ubyte)]
lib.quat_image_get_last_error_message.restype = ctypes.c_char_p
lib.quat_image_free_error_message.argtypes = [ctypes.c_char_p]

class QuatImage:
    def __init__(self, w, h):
        self.handle = lib.quat_image_new(w, h)
        if not self.handle: self._err()
    def __del__(self):
        if hasattr(self, 'handle') and self.handle: lib.quat_image_free(self.handle)
    def set_pixel(self, x, y, v):
        if lib.quat_image_set_pixel(self.handle, x, y, v) != 0: self._err()
    def get_pixel(self, x, y):
        val = ctypes.c_ubyte()
        if lib.quat_image_get_pixel(self.handle, x, y, ctypes.byref(val)) != 0: self._err()
        return val.value
    def _err(self):
        ptr = lib.quat_image_get_last_error_message()
        msg = ptr.decode('utf-8') if ptr else "Error"
        if ptr: lib.quat_image_free_error_message(ptr)
        raise RuntimeError(msg)
```