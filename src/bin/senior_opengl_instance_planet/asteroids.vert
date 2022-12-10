#version 330 core
in vec3 position;
in vec2 texture;
in mat4 model;

out VS_OUT {
    vec2 texCoords;
} vs_out;

uniform mat4 projection;
uniform mat4 view;

void main()
{
    vs_out.texCoords = texture;
    gl_Position = projection * view * model * vec4(position, 1.0);
}