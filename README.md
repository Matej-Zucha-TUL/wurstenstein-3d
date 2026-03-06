# ITE/PG2 2025/2026 - Semestrální práce 

```rust
use Zumepro::{MichalProcházka, MatějŽucha};
```

Fun learning materials: https://github.com/afaber999/rust_learn_opengl_glow/blob/main/src





# ✅ 1st lab: Project setup
- [x] setting up the compiler, OpenGL Hello world
  - We use Rust with the Glow and egui libraries





# ⏱️ 2nd lab: OpenGL, FPS, VSync

## ✅ Task 1 - allow OpenGL debugging

- [x] check for OpenGL debug extension (if you succeeded to open GL context version 4.6 in last lab, it should be present)
  - egui does this for us
- [x] create debug callback (copy from lecture)
  - egui does this for us
- [x] activate debug output. If it is too noisy, use filter to suppress notifications, etc.
  - well, you guessed it... egui does this for us

## ✅ Task 2 - use GLFW (a bit more) safely

- [x] create and register GLFW error callback to handle errors during library initialization
  - .unwrap() into crash = profit
- [x] add proper error and quit handling (in C++ use exceptions)
  - .unwrap() strikes again 

## ✅ Task 3 - Modify (extend) your app

- [x] measure FPS (frame per second)
  - [x] display FPS as a window title
- [x] create and register additional callbacks

## ⏱️ Task 4 - implement VSync toggle

- [x] create key callback to toggle vsync
  - [ ] modify window title to show vsync off, on
  - [x] note: see the FPS difference  
- [ ] create and use ~JSON~ TOML config file to set initial size of the GLFW window
  - or none, as we use tiling window managers lol





# ⏱️ 3rd lab: GUI - User Interface

## ⏱️ Task 1: Implement GUI, mouse cursor catch, hidden window during startup

- [x] 1. When in fullscreen, you can not see FPS values in window title or debug output in console window. To overcome this, implement simple GUI using ImGUI library.
  - we decided to use egui

- [ ] 2. When the app is in windowed mode and cursor is enabled, it can leave the app window, and mouse events will not be received. You can disable cursor - but then you can not click on close button in application title bar.
  - [ ] see how you can capture and release mouse button
  - [ ] you can modify the application logic: cursor can be released by e.g. TAB key, or first ESC (second ESC will terminate), etc.

- [ ] 3. During start-up, loading assets (models, textures, compiling shaders etc.) can take a long time. In the meantime, application window does not respond and is empty - and this could disturb the user (may think, that the app hang...).
  - [ ] see how to hide the window during initialization
  - [ ] the other idea is to display some kind of loadscreen. That would also require some init & draw, so you must choose it carefully.

## ⏱️ Task 2: Implement toggle Window Mode <---> Full-screeen mode

- [x] properly save and restore window position and size, including multimonitor setup.
  - not possible on Wayland lol





# ⏱️ 4th lab: Implementation of generic resource load

## ⏱️ Task 1: Generic shader loader - directory __01 shader class__

- [x] Current state: shader hard-coded into .rs source file as a string
  - never happened lol
- [ ] Target: two external files (suffixes .vert for vertex shader, .frag for fragment shader; if you use different suffixes, GLSL plugin can not perform syntax check and highlighting). Implemented class, that will load both shader files files, and get shader program ready. Create functions to set uniform variables for CPU-GPU communication.
  - kind of done already, we just need to wrap everything into a nice class

## ⏱️ Task 2: Explore directory __02 shader examples__

- [ ] see directory description, and explore the shader functionality
- [ ] some functions will be used in following lectures

## ⏱️ Task 3: Simple generic model loader - directory __03 vertex-mesh-model class__

- [ ] Current state: triangle vertex data are hard-coded into source code
- [ ] Resources: in subdirectory of _04 loading assets_ you can find file __triangle.obj__ with following content:
- [ ] Target: implemented class that will load .OBJ file, parse the content and create VAO, VBO, set parameters etc., so the triangle data will be stored outside the source code. Use also __EBO__ (see lectures) for indirect vertex addressing.

- [ ] Modify your __assets.hpp__, so that vertex structure contains normal and texture coordinate.
- [ ] Copy partially implemented classes (Mesh.hpp, Model.hpp) into your project directory and add to project.
- [ ] Explore OBJloader.cpp and OBJloader.hpp from __04 loading assets__, that can  load OBJ file. The loader is simple and limited: it expects, that model in .OBJ file __allways contains texture coordinates and normals, and uses triangles__.
- [ ] Use the lecture to implement missing parts - marked as "TODO". Fully setup and initialize VAO. __Use DSA.__
- [ ] Draw the triangle.

### ⏱️ Task 3a (OPTIONAL): Modify and extend the functionality of OBJ loader

- [ ] Loader expects triangles. Modify it, so that if it finds Quad, it will break it in two triangles.
- [ ] Loader expects normals coordinates. Modify it, so that if no normals are found, it will calculate it: for triangle, it is
- [ ] Loader expects texture coordinates. Modify it, so that if no texcoords are found, it will provide fake fixed coordinate glm::vec2(0.0f)

### ⏱️ Task 3b (superOPTIONAL): Meshlab

- [ ] Download Meshlab, load some model, try to convert it to .OBJ format. Try other functions of Meshlab, like increasing/decreasing triangle count. This can be used to implement simple LOD (Level Of Detail).
