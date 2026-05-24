# Wurstenstein 3D - ITE/PG2 2025/2026 - Semestrální práce 

```rust
use Zumepro::{MichalProcházka, MatějŽucha};
```

## Game controls

- WASD - walk around
- Space - jump
- V - toggle Vsync
- F - toggle fullscreen
- O - take screenshot
- L - unlock cursor, shows debug window
- R - respawn if game finished
- P - toggle freeform camera (for testing)
- Mouse movement - look around
- Left mouse button - shoot
- Right mouse button - enable flashlight
- Mouse scroll wheel - change FOV

## Final criteria:

- Default value is 100 points
- It is required to send full project and installation procedures in advance (using gitlab.tul.cz, GitHub, Gitlab, etc.)

### Essentials:

Each missing (non-functional) Essential nets -25 points (partial functionality => partial decrenent)

- [X] 3D GL Core profile + shaders version 4.6
- [X] GL debug enabled
- [X] ~JSON~ TOML config file
- [X] high performance => at least 60 FPS (display FPS)
- [X] allow VSync control
- [X] allow antialiasing
- [X] fullscreen vs. windowed switching
- [X] restore window position & size
- [X] event processing (camera, object, app behaviour, ...): mouse (both axes, wheel), keyboard
- [X] multiple different independently moving 3D models, at least two loaded from file
- [X] at least three different textures (or subtextures from texture atlas etc.)
- [X] lighting model, all basic lights types (1x anbient, min. 1x directional, min. 2x point, min. Ix reflector; at least two are moving independently
- [X] correct full alpha scale transparency (at least two transparent objects; NOT if(alpha<0.1) {discard;} )
- [X] correct collisions

### Extras:

Each working Extra nets +10 points

- [ ] height map textured by height t proper player height coords
- [X] audio (better than just background)
- [ ] particle effects
- [ ] scripting (useful)
- [X] some other nice complicated effect..
  - does the starfield count? xD

### Instafail

Any of these criteria met will result in instant rejection

- [X] use GLUT
- [X] use GL compatible profile
- [X] no DSA (direct state access)

## Lab logs (not relevant to the final project):

### ✅ 1st lab: Project setup
#### ✔️ Task 1 - Setting up the compiler, OpenGL Hello world
  - We use the `Rust` programming language with `glow` and `egui` libraries





### ✅ 2nd lab: OpenGL, FPS, VSync

#### ✔️ Task 1 - allow OpenGL debugging

- [x] Check for OpenGL debug extension (if you succeeded to open GL context version 4.6 in last lab, it should be present)
- [x] Create debug callback (copy from lecture)
- [x] Activate debug output. If it is too noisy, use filter to suppress notifications, etc.

#### ✔️ Task 2 - use GLFW (a bit more) safely

- [x] Create and register GLFW error callback to handle errors during library initialization
  - .unwrap() into crash = profit
- [x] Add proper error and quit handling (in C++ use exceptions)
  - .unwrap() strikes again 

#### ✔️ Task 3 - Modify (extend) your app

- [x] Measure FPS (frame per second)
  - [x] Display FPS as a window title
- [x] Create and register additional callbacks

#### ✔️ Task 4 - implement VSync toggle

- [x] Create key callback to toggle vsync
  - [x] Modify window title to show vsync off, on
  - [x] Note: see the FPS difference  
- [x] Create and use ~JSON~ TOML config file to set initial size of the GLFW window
  -  ̶O̶r̶ ̶n̶o̶n̶e̶,̶ ̶a̶s̶ ̶w̶e̶ ̶u̶s̶e̶ ̶t̶i̶l̶i̶n̶g̶ ̶w̶i̶n̶d̶o̶w̶ ̶m̶a̶n̶a̶g̶e̶r̶s̶ ̶l̶o̶l̶  I use Plasma @home sometimes, I will implement it





### ✅ 3rd lab: GUI - User Interface

#### ✔️ Task 1 - Implement GUI, mouse cursor catch, hidden window during startup

- [x] 1. When in fullscreen, you can not see FPS values in window title or debug output in console window. To overcome this, implement simple GUI using ImGUI library.
  - we decided to use egui

- [x] 2. When the app is in windowed mode and cursor is enabled, it can leave the app window, and mouse events will not be received. You can disable cursor - but then you can not click on close button in application title bar.
  - [x] See how you can capture and release mouse button
  - [x] You can modify the application logic: cursor can be released by e.g. TAB key, or first ESC (second ESC will terminate), etc.

- [x] 3. During start-up, loading assets (models, textures, compiling shaders etc.) can take a long time. In the meantime, application window does not respond and is empty - and this could disturb the user (may think, that the app hang...).
  - [x] See how to hide the window during initialization
  - [ ] ~The other idea is to display some kind of loadscreen. That would also require some init & draw, so you must choose it carefully.~
    - nah

#### ✔️ Task 2 - Implement toggle Window Mode <---> Full-screeen mode

- [X] ~Properly save and restore window position and size, including multimonitor setup.~
  - Not possible on Wayland lol
  - We can give it a shot on X11 (big sad)
    - Does not work in dwm either. Also, this is just terrible, why would anyone want this?
    - nevermind, got it working and tested in GNUstep lol





### ✅ 4th lab: Implementation of generic resource load

#### ✔️ Task 1 - Generic shader loader - directory __01 shader class__

- [x] Current state: shader hard-coded into .rs source file as a string
  - Never happened lol
- [X] Target: two external files (suffixes .vert for vertex shader, .frag for fragment shader; if you use different suffixes, GLSL plugin can not perform syntax check and highlighting). Implemented class, that will load both shader files files, and get shader program ready. Create functions to set uniform variables for CPU-GPU communication.
  - Kind of done already, we just need to wrap everything into a nice class

#### ✔️ Task 2 - Explore directory __02 shader examples__

- [X] See directory description, and explore the shader functionality
- [X] Some functions will be used in following lectures

#### ✔️ Task 3 - Simple generic model loader - directory __03 vertex-mesh-model class__

- [X] Current state: triangle vertex data are hard-coded into source code
- [X] Resources: in subdirectory of _04 loading assets_ you can find file __triangle.obj__ with following content:
- [X] Target: implemented class that will load .OBJ file, parse the content and create VAO, VBO, set parameters etc., so the triangle data will be stored outside the source code. Use also __EBO__ (see lectures) for indirect vertex addressing.

- [X] Modify your __assets.hpp__, so that vertex structure contains normal and texture coordinate.
- [X] Copy partially implemented classes (Mesh.hpp, Model.hpp) into your project directory and add to project.
- [X] Explore OBJloader.cpp and OBJloader.hpp from __04 loading assets__, that can  load OBJ file. The loader is simple and limited: it expects, that model in .OBJ file __always contains texture coordinates and normals, and uses triangles__.
- [X] Use the lecture to implement missing parts - marked as "TODO". Fully setup and initialize VAO. __Use DSA.__
- [X] Draw the triangle.

##### ⏱️ Task 3a (OPTIONAL) - Modify and extend the functionality of OBJ loader

- [X] Loader expects triangles. Modify it, so that if it finds Quad, it will break it in two triangles.
- [ ] Loader expects normals coordinates. Modify it, so that if no normals are found, it will calculate it: for triangle, it is
- [ ] Loader expects texture coordinates. Modify it, so that if no texcoords are found, it will provide fake fixed coordinate glm::vec2(0.0f)

##### ✔️ Task 3b (superOPTIONAL) - Meshlab

- [X] Download Meshlab, load some model, try to convert it to .OBJ format. Try other functions of Meshlab, like increasing/decreasing triangle count. This can be used to implement simple LOD (Level Of Detail).





### ✅ 5th lab: This lab was canceled





### ✅ 6th lab: 

#### ✔️ Task 1 - See how to set all transformations

- [X] Read _transform00-basic explanation.cpp_. This file is __NOT__ meant to be included in your project, it is only explanation.
See, how __model__, __view__, __projection__ and __viewport__ transformations are set, and how they are received in vertex shader __basic_core.vert__

#### ✔️ Task 2: Implementing transformations in our App

To use transformations, we must add code into various parts of our program. Modify __Model__ and __Mesh__ classes, so that they are cabable to draw itself with transformations (see _Model-extended.h_ for inspiration). Modify callbacks and main app loop to use transformations.

- [X] In vertex shader, transformation matrices are defaulted to diagonal matrix = identity. This is the safe default - if you do not set some matrix, it will not change passed values in any way. That means, you can implement transformations step-by-step and the application will gradually improve.
  1) implement model matrix setting: try to move the object using time. Modify model matrix of the object and use shader.setUniform() to set uniform variable in a shader.
  2) implement view matrix setting: implement camera movement using key polling, create view matrix using glm::lookAt()
  3) when window size or FOV changes, set projection matrix and viewport (do not forget to set it at the app start)
  4) implement mouse control of the camera
- [X] create new variables in _private_ to store projection matrix and related values (e.g. fov). There is no need to recompute projection matrix each frame, it should be updated only by:
  - [X] resizing window (callback)
  - [X] changing field-of-view (callback)
- [X] view matrix is computed from position and orientation of the viewer (i.e. player i.e. camera) - it is probably changing in each frame. For now, it is hardcoded, we will create dynamic camera in Task 3.
- [X] model matrix can be set by several different ways, see examples

#### ✔️ Task 3: Implement camera, that can move and look around (keyboard + mouse)

Camera rotation (looking around) and movement must be handled differently: read _transform.cpp_ to get an idea, how to implement cursorPositionCallback() for look-around and direct key state polling for camera movement.

- [X] finish implementation of __camera.hpp__
- [X] implement:
  - [X] free floating camera
  - [X] POV camera (locked to some visible object)

#### ⏱️ Task 4 (OPTIONAL): changing movement speed

Most simple movement is _key=move_ approach, or _shift+key=faster_move_. That is unrealistic. You can implement more realistic approach: acceleration + friction (for movement on the ground) or drag (movement in the air) - or combination.

```C++
// objects moving on the ground
if (on_ground) {
    const float friction_coefficient = 5.0f; // decceleration
    glm::vec3 horizontal_velocity(velocity.x, 0.0f, velocity.z);
    glm::vec3 horizontal_slowdown = -horizontal_velocity * friction_coefficient * deltaTime;
    velocity += horizontal_slowdown;
}
```

Drag is similar, but weaker and deccelerates at all times...

```C++
// gravity
glm::vec3 acceleration{0.0f};
if (affected_by_gravity) {
    acceleration.y += -9.81f;
}
// keys:
direction = ... // get desired direction unit vector from pressed keys

if (glfwGetKey(window, GLFW_KEY_LEFT_SHIFT)...) // running
    accel_multiplier = 2.0f;
else
    accel_multiplier = 1.0f;

if (glfwGetKey(...) ... && (position.y == 0.0f) ...) // jump, no double jump
   velocity.y += 10.0f; 

acceleration += direction * player_acceleration * accel_multiplier;
velocity += acceleration * deltaTime;
position += velocity * deltaTime;
```

There are even more realistic (complicated) models (object mass, applied force, drag force dependent on speed etc.).





### ✅ 7th lab: Textures

#### ✔️ Task 1: Antialiasing

- [X] Use GLFW hint to initialize antialiasing with level 4. Use glEnable/glDisable(GL_MULTISAMPLE) to compare FPS with/without antialiasing.

#### ✔️ Task 2: Screenshot

- [X] Implement screenshot functionality. Create two screenshots - with and without antialiasing. Open them in an image viewer, zoom in and compare image quality.

#### ✔️ Task 3: Explore the source code of the GL demonstration

#### ✔️ Task 4: Display textured object

- [X] implement: loading texture from file
- [X] do __NOT__ use stbi_image library! We already have OpenCV...

- [X] use example to extend functionality of Model+Mesh class
- [X] use shader example to implement texture support
- [X] load OBJ model with texture coordinates
- [X] ...enjoy

### ✅ 8th lab: Lighting

#### ✔️ Task 1: Implement most simple light source - directional light (sun)

- [x] implement Phong lighting model for directional light
  - [x] start with shaders: create static light source with hardcoded settings - create light model parameters as __uniforms with default value__. This will later allow you to override them from CPU side (C++).
  - [x] if it works for default values, go to C++ and create data structures for light parameters. Modify params in time (moving sun, sun changing color, etc.) and update uniforms dynamically.

#### ✔️ Task 2: Implement at least 3 different point lights

- [X] Create at least 3 point lights with different parameters.
- [X] Move lights independently: light position is just point in space. Create model matrix for some model transformations (translate, rotate, scale), and transform original position of the light (point) to new position by matrix multiplication.

#### ✔️ Task 3: Implement at least one spot light

- [X] Implement spot light = reflector light source, for example as a light attached to the camera (headlight).





### ✅ 9th lab: Transparency, Physics

#### ✔️ Task 1: Transparency

Allow the App to correctly draw transparent objects.

- [X] create at least 3 (semi)transparent objects. Use either material with A<1.0 or texture with alpha channel.
- [X] use correct, full scale transparency (NOT if(alpha<0.1) {discard;} )

#### ✔️ Task 2: Implement collision detection

The very basic collision detection is to disallow the player to leave the map. Better version should be implemented.

- [X] implement at least two of
  - [ ] correct behaviour of hit-wall (sliding)
  - [X] detect hit of some object with projectile (touch player-object)
  - [X] detect hit (touch) of the enemy with player
  - [X] some other idea... (collision with floor)

#### ✔️ Task 3: Implement simple particle effect

Particles exists only for short time, no collision detection is usually performed. You can dynamically add simple object to the scene, and after short moment (lifetime of whole particle effect) remove it. You can implement particles with several simple methods:

- [X] point cloud (GL_POINT)
- [ ] small non-textured mesh (minimal mesh is tetraedron - four vertices forming GL_TRIANGLE_STRIP)
- [ ] (OPTIONAL) point sprites





