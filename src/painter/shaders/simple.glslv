#version 150 core

in vec4 a_Pos;
in vec4 a_Color;

uniform Transform {
    float u_ScreenRatio;
};

out vec4 v_Color;

void main() {
    v_Color = a_Color;
    gl_Position = a_Pos;
    gl_Position[0] /= u_ScreenRatio;
}