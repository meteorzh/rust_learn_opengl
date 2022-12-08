#version 330 core
in vec3 position;
in vec2 texture;

out VS_OUT {
    vec2 texCoords;
} vs_out;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main()
{
    vs_out.texCoords = texture;
    gl_Position = projection * view * model * vec4(position, 1.0);
}