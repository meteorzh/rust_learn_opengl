#version 330 core
out vec3 GPosition;
out vec3 GNormal;
out vec3 GColorSpecular;

in vec2 TexCoords;
in vec3 FragPos;
in vec3 Normal;

void main()
{    
    // store the fragment position vector in the first gbuffer texture
    GPosition = FragPos;
    // also store the per-fragment normals into the gbuffer
    GNormal = normalize(Normal);
    // and the diffuse per-fragment color
    GColorSpecular.rgb = vec3(0.95);
}