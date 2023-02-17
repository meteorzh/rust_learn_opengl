#version 330 core
in vec3 position;

out vec3 TexCoords;

uniform mat4 projection;
uniform mat4 view;

void main()
{
    TexCoords = position;
    vec4 pos = projection * view * vec4(position, 1.0);
    // 顶点着色器之后的透视除法计算深度值，此处保证，天空盒计算后的深度值始终为1，即深度值始终最大（同时深度函数需要改为<=）
    gl_Position = pos.xyww;
}