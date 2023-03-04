#version 330 core
in vec3 position;

uniform mat4 capture_projection;
uniform mat4 view;

out vec3 WorldPos;

void main()
{
    WorldPos = position;
    gl_Position = capture_projection * view * vec4(position, 1.0);
}