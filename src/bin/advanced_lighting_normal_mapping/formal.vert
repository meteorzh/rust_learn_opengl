#version 330 core
in vec3 position;
in vec3 normal;
in vec2 texture;

// declare an interface block; see 'Advanced GLSL' for what these are.
out VS_OUT {
    vec3 FragPos;
    vec3 Normal;
    vec2 TexCoords;
} vs_out;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main()
{
    vs_out.FragPos = position;
    vs_out.Normal = normal;
    vs_out.TexCoords = texture;
    gl_Position = projection * view * model * vec4(position, 1.0);
}