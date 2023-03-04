#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices=18) out;

uniform mat4 cube_views[6];
uniform mat4 capture_projection;

out vec3 WorldPos;
out int face;

void main()
{
    for(int f = 0; f < 6; ++f)
    {
        gl_Layer = f; // built-in variable that specifies to which face we render.
        face = f;
        for(int i = 0; i < 3; ++i) // for each triangle vertex
        {
            WorldPos = gl_in[i].gl_Position.xyz;
            gl_Position = capture_projection * cube_views[f] * gl_in[i].gl_Position;
            // gl_Position = cubeMatrixes[face] * gl_in[i].gl_Position;
            // FragPos = gl_Position;
            EmitVertex();
        }
        EndPrimitive();
    }
}