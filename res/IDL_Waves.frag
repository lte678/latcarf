// The MIT License (MIT)
// 
// Copyright (c) 2015 kbinani
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// Thank you kbinani for the colormaps!

float colormap_f(float x, float phase) {
	const float pi = 3.141592653589793238462643383279502884197169399;
	const float a = 126.9634465941118;
	const float b = 1.011727672706345;
	const float c = 0.0038512319231245;
	const float d = 127.5277540583575;
	return a * sin(2.0 * pi / b * x + 2.0 * pi * (c + phase)) + d;
}

float colormap_red(float x) {
	return colormap_f(x, 0.5);
}

float colormap_green(float x) {
	const float pi = 3.141592653589793238462643383279502884197169399;
	const float a = 63.19460736097507;
	const float b = 0.06323746667143024;
	const float c = 0.06208443629833329;
	const float d = 96.56305326777574;
	return a * sin(2.0 * pi / b * x + 2.0 * pi * c) + d;
}

float colormap_blue(float x) {
	return colormap_f(x, 0.0);
}

vec4 colormap(float x) {
	float r = clamp(colormap_red(x) / 255.0, 0.0, 1.0);
	float g = clamp(colormap_green(x) / 255.0, 0.0, 1.0);
	float b = clamp(colormap_blue(x) / 255.0, 0.0, 1.0);
	return vec4(r, g, b, 1.0);
}