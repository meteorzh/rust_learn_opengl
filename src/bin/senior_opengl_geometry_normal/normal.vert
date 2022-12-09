#version 330 core
in vec3 position;
in vec3 normal;

out VS_OUT {
    vec3 Normal;
} vs_out;

uniform mat4 view;
uniform mat4 model;

void main()
{
    gl_Position = view * model * vec4(position, 1.0);
    mat3 normalMatrix = mat3(transpose(inverse(view * model)));
    vs_out.Normal = normalize(vec3(vec4(normalMatrix * normal, 0.0)));
}