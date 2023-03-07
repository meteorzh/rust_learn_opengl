#version 330 core
out vec4 FragColor;

in vec3 FragPos;

uniform samplerCube image;
uniform float exposure;

const float gamma = 2.2;

void main()
{
    vec3 hdrColor = texture(image, FragPos).rgb;

    vec3 result = vec3(1.0) - exp(-hdrColor * exposure);
    // also gamma correct while we're at it
    result = pow(result, vec3(1.0 / gamma));
    FragColor = vec4(result, 1.0);
    // FragColor = texture(image, FragPos);
}