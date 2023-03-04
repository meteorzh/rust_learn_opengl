#version 330 core
in vec3 position;
in vec2 texture;

uniform mat4 projection;
uniform mat4 view;

out vec3 FragPos;

void main()
{
    FragPos = position;
    gl_Position = (projection * view * vec4(position, 1.0)).xyww;
}