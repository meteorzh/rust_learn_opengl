#version 330 core
in vec3 position;
in vec2 texture;

uniform mat4 projection;
uniform mat4 view;

out vec3 FragPos;

void main()
{
    FragPos = position;
    mat4 rotView = mat4(mat3(view));
    vec4 clipPos = projection * rotView * vec4(position, 1.0);

    gl_Position = clipPos.xyww;
}