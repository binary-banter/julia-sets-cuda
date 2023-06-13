extern "C" __global__ void julia(const int width,
                                 const int height,
                                 const float c_x,
                                 const float c_y,
                                 const int iterations,
                                 const float top,
                                 const float bottom,
                                 const float left,
                                 const float right,
                                 unsigned char *d_colors,
                                 const unsigned char *d_color_map) {
    unsigned int idx = threadIdx.x + blockIdx.x * blockDim.x;
    unsigned int idy = threadIdx.y + blockIdx.y * blockDim.y;

    if (idx < width && idy < height) {
        float x = left + (right - left) / (float) width * (float) idx;
        float y = top - (top - bottom) / (float) height * (float) idy;
        float x_temp;
        int steps = -1;

        // check if escaped
        for (int i = 0; i < iterations; i++) {
            if (x * x + y * y > 4) {
                steps = i;
                break;
            } else {
                x_temp = x * x - y * y;
                y = 2 * x * y + c_y;
                x = x_temp + c_x;
            }
        }

        // set colors
        unsigned int idc = (idy * width + idx) * 3;

        if (steps == -1) {
            d_colors[idc] = 0;
            d_colors[idc + 1] = 0;
            d_colors[idc + 2] = 0;
        } else {
            d_colors[idc] = d_color_map[steps * 3];
            d_colors[idc + 1] = d_color_map[steps * 3 + 1];
            d_colors[idc + 2] = d_color_map[steps * 3 + 2];
        }
    }
}