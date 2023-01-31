#version 420 core

layout(location = 0) in vec3 vPosition;
layout(location = 1) in vec3 vColor;

out vec3 color;

uniform float xPosition;
uniform float yPosition;

void main() {
    color = vColor;
    gl_Position = vec4((vPosition.x + xPosition), (vPosition.y + yPosition), vPosition.z, 1.0);
}