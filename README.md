# ITE/PG2 2025/2026 - Semestrální práce 

```rust
use Zumepro::{MichalProcházka, MatějŽucha};
```

Fun learning materials: https://github.com/afaber999/rust_learn_opengl_glow/blob/main/src





# ✅ 1st lab: Project setup
## ✔️ Task 1 - Setting up the compiler, OpenGL Hello world
  - We use the `Rust` programming language with `glow` and `egui` libraries





# ✅ 2nd lab: OpenGL, FPS, VSync

## ✔️ Task 1 - allow OpenGL debugging

- [x] Check for OpenGL debug extension (if you succeeded to open GL context version 4.6 in last lab, it should be present)
  - egui does this for us
- [x] Create debug callback (copy from lecture)
  - egui does this for us
- [x] Activate debug output. If it is too noisy, use filter to suppress notifications, etc.
  - well, you guessed it... egui does this for us

## ✔️ Task 2 - use GLFW (a bit more) safely

- [x] Create and register GLFW error callback to handle errors during library initialization
  - .unwrap() into crash = profit
- [x] Add proper error and quit handling (in C++ use exceptions)
  - .unwrap() strikes again 

## ✔️ Task 3 - Modify (extend) your app

- [x] Measure FPS (frame per second)
  - [x] Display FPS as a window title
- [x] Create and register additional callbacks

## ✔️ Task 4 - implement VSync toggle

- [x] Create key callback to toggle vsync
  - [x] Modify window title to show vsync off, on
  - [x] Note: see the FPS difference  
- [x] Create and use ~JSON~ TOML config file to set initial size of the GLFW window
  -  ̶O̶r̶ ̶n̶o̶n̶e̶,̶ ̶a̶s̶ ̶w̶e̶ ̶u̶s̶e̶ ̶t̶i̶l̶i̶n̶g̶ ̶w̶i̶n̶d̶o̶w̶ ̶m̶a̶n̶a̶g̶e̶r̶s̶ ̶l̶o̶l̶  I use Plasma @home sometimes, I will implement it





# ❌ 3rd lab: GUI - User Interface

## ✔️ Task 1 - Implement GUI, mouse cursor catch, hidden window during startup

- [x] 1. When in fullscreen, you can not see FPS values in window title or debug output in console window. To overcome this, implement simple GUI using ImGUI library.
  - we decided to use egui

- [x] 2. When the app is in windowed mode and cursor is enabled, it can leave the app window, and mouse events will not be received. You can disable cursor - but then you can not click on close button in application title bar.
  - [x] See how you can capture and release mouse button
  - [x] You can modify the application logic: cursor can be released by e.g. TAB key, or first ESC (second ESC will terminate), etc.

- [x] 3. During start-up, loading assets (models, textures, compiling shaders etc.) can take a long time. In the meantime, application window does not respond and is empty - and this could disturb the user (may think, that the app hang...).
  - [x] See how to hide the window during initialization
  - [ ] ~The other idea is to display some kind of loadscreen. That would also require some init & draw, so you must choose it carefully.~
    - nah fuck that

## ⏱️ Task 2 - Implement toggle Window Mode <---> Full-screeen mode

- [ ] Properly save and restore window position and size, including multimonitor setup.
  - Not possible on Wayland lol
  - We can give it a shot on X11 (big sad)





# ❌ 4th lab: Implementation of generic resource load

## ✔️ Task 1 - Generic shader loader - directory __01 shader class__

- [x] Current state: shader hard-coded into .rs source file as a string
  - Never happened lol
- [X] Target: two external files (suffixes .vert for vertex shader, .frag for fragment shader; if you use different suffixes, GLSL plugin can not perform syntax check and highlighting). Implemented class, that will load both shader files files, and get shader program ready. Create functions to set uniform variables for CPU-GPU communication.
  - Kind of done already, we just need to wrap everything into a nice class

## ⏱️ Task 2 - Explore directory __02 shader examples__

- [ ] See directory description, and explore the shader functionality
- [ ] Some functions will be used in following lectures

## ⏱️ Task 3 - Simple generic model loader - directory __03 vertex-mesh-model class__

- [ ] Current state: triangle vertex data are hard-coded into source code
- [ ] Resources: in subdirectory of _04 loading assets_ you can find file __triangle.obj__ with following content:
- [ ] Target: implemented class that will load .OBJ file, parse the content and create VAO, VBO, set parameters etc., so the triangle data will be stored outside the source code. Use also __EBO__ (see lectures) for indirect vertex addressing.

- [ ] Modify your __assets.hpp__, so that vertex structure contains normal and texture coordinate.
- [ ] Copy partially implemented classes (Mesh.hpp, Model.hpp) into your project directory and add to project.
- [ ] Explore OBJloader.cpp and OBJloader.hpp from __04 loading assets__, that can  load OBJ file. The loader is simple and limited: it expects, that model in .OBJ file __always contains texture coordinates and normals, and uses triangles__.
- [ ] Use the lecture to implement missing parts - marked as "TODO". Fully setup and initialize VAO. __Use DSA.__
- [ ] Draw the triangle.

### ⏱️ Task 3a (OPTIONAL) - Modify and extend the functionality of OBJ loader

- [ ] Loader expects triangles. Modify it, so that if it finds Quad, it will break it in two triangles.
- [ ] Loader expects normals coordinates. Modify it, so that if no normals are found, it will calculate it: for triangle, it is
- [ ] Loader expects texture coordinates. Modify it, so that if no texcoords are found, it will provide fake fixed coordinate glm::vec2(0.0f)

### ⏱️ Task 3b (superOPTIONAL) - Meshlab

- [ ] Download Meshlab, load some model, try to convert it to .OBJ format. Try other functions of Meshlab, like increasing/decreasing triangle count. This can be used to implement simple LOD (Level Of Detail).
