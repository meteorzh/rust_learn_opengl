#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D texture1;

void main()
{
    // 丢弃草纹理的透明部分
    // vec4 texColor = texture(texture1, TexCoords);
    // if(texColor.a < 0.1)
    //     discard;
    // FragColor = texColor;

    FragColor = texture(texture1, TexCoords);
}