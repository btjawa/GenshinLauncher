fn main() {
    cc::Build::new()
        .cpp(true)
        .file("./src/unlock_fps/unlock_fps.cpp")
        .include("./src/unlock_fps/unlock_fps.h")
        .compile("unlock_fps");
    tauri_build::build();
}
