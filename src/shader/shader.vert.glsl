#version 450

layout(location = 0) in vec2 aPosition; // Input vertex position

void main() {
    gl_Position = vec4(aPosition, 0.0, 1.0); // Set the vertex position in clip space
}
