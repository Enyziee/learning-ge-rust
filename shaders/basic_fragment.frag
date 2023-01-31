#version 420 core

in vec3 color;
out vec4 FragColor;

uniform float xPosition;
uniform float yPosition;

void main() {
    FragColor = vec4(color, 1.0f);
}