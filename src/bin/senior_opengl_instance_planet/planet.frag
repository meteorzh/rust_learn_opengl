#version 330 core

in VS_OUT {
    vec2 texCoords;
} gs_in;

out vec4 FragColor;

uniform sampler2D texture_diffuse;

void main()
{
    FragColor = texture(texture_diffuse, gs_in.texCoords);
}