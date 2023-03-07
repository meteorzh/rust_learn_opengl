#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D image;
uniform float exposure;
const float gamma = 2.2;

void main()
{
    // FragColor = vec4(texture(image, TexCoords).rgb, 1.0);

    vec3 hdrColor = texture(image, TexCoords).rgb;
    vec3 result = vec3(1.0) - exp(-hdrColor * exposure);
    // also gamma correct while we're at it
    result = pow(result, vec3(1.0 / gamma));
    FragColor = vec4(result, 1.0);
}