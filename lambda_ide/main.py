from __future__ import absolute_import
import ctypes

from sdl2 import *
import imgui
import OpenGL.GL as gl

from imgui.integrations.opengl import FixedPipelineRenderer


class FixedPipelineSDL2Renderer(FixedPipelineRenderer):
    """Basic SDL2 integration implementation."""
    MOUSE_WHEEL_OFFSET_SCALE = 0.5

    def __init__(self, window):
        super(FixedPipelineSDL2Renderer, self).__init__()
        self.window = window

        self._mouse_pressed = [False, False, False]
        self._mouse_wheel = 0.0
        self._gui_time = None

        width_ptr = ctypes.pointer(ctypes.c_int(0))
        height_ptr = ctypes.pointer(ctypes.c_int(0))
        SDL_GetWindowSize(self.window, width_ptr, height_ptr)

        self.io.display_size = width_ptr[0], height_ptr[0]

        self._map_keys()

    def _map_keys(self):
        key_map = self.io.key_map

        key_map[imgui.KEY_TAB] = SDLK_TAB
        key_map[imgui.KEY_LEFT_ARROW] = SDL_SCANCODE_LEFT
        key_map[imgui.KEY_RIGHT_ARROW] = SDL_SCANCODE_RIGHT
        key_map[imgui.KEY_UP_ARROW] = SDL_SCANCODE_UP
        key_map[imgui.KEY_DOWN_ARROW] = SDL_SCANCODE_DOWN
        key_map[imgui.KEY_PAGE_UP] = SDL_SCANCODE_PAGEUP
        key_map[imgui.KEY_PAGE_DOWN] = SDL_SCANCODE_PAGEDOWN
        key_map[imgui.KEY_HOME] = SDL_SCANCODE_HOME
        key_map[imgui.KEY_END] = SDL_SCANCODE_END
        key_map[imgui.KEY_DELETE] = SDLK_DELETE
        key_map[imgui.KEY_BACKSPACE] = SDLK_BACKSPACE
        key_map[imgui.KEY_ENTER] = SDLK_RETURN
        key_map[imgui.KEY_ESCAPE] = SDLK_ESCAPE
        key_map[imgui.KEY_A] = SDLK_a
        key_map[imgui.KEY_C] = SDLK_c
        key_map[imgui.KEY_V] = SDLK_v
        key_map[imgui.KEY_X] = SDLK_x
        key_map[imgui.KEY_Y] = SDLK_y
        key_map[imgui.KEY_Z] = SDLK_z

    def process_event(self, event):
        io = self.io

        if event.type == SDL_MOUSEWHEEL:
            self._mouse_wheel = event.wheel.y * self.MOUSE_WHEEL_OFFSET_SCALE
            return True

        if event.type == SDL_MOUSEBUTTONDOWN:
            if event.button.button == SDL_BUTTON_LEFT:
                self._mouse_pressed[0] = True
            if event.button.button == SDL_BUTTON_RIGHT:
                self._mouse_pressed[1] = True
            if event.button.button == SDL_BUTTON_MIDDLE:
                self._mouse_pressed[2] = True
            return True

        if event.type == SDL_KEYUP or event.type == SDL_KEYDOWN:
            key = event.key.keysym.sym & ~SDLK_SCANCODE_MASK

            if key < SDL_NUM_SCANCODES:
                io.keys_down[key] = event.type == SDL_KEYDOWN

            io.key_shift = ((SDL_GetModState() & KMOD_SHIFT) != 0)
            io.key_ctrl = ((SDL_GetModState() & KMOD_CTRL) != 0)
            io.key_alt = ((SDL_GetModState() & KMOD_ALT) != 0)

            return True

        if event.type == SDL_TEXTINPUT:
            for char in event.text.text.decode('utf-8'):
                io.add_input_character(ord(char))
            return True

    def process_inputs(self):
        io = imgui.get_io()

        s_w = ctypes.pointer(ctypes.c_int(0))
        s_h = ctypes.pointer(ctypes.c_int(0))
        SDL_GetWindowSize(self.window, s_w, s_h)
        w = s_w.contents.value
        h = s_h.contents.value

        io.display_size = w, h
        io.display_fb_scale = 1, 1

        current_time = SDL_GetTicks() / 1000.0

        if self._gui_time:
            self.io.delta_time = current_time - self._gui_time
        else:
            self.io.delta_time = 1. / 60.
        self._gui_time = current_time

        mx = ctypes.pointer(ctypes.c_int(0))
        my = ctypes.pointer(ctypes.c_int(0))
        mouse_mask = SDL_GetMouseState(mx, my)

        if SDL_GetWindowFlags(self.window) & SDL_WINDOW_MOUSE_FOCUS:
            io.mouse_pos = mx.contents.value, my.contents.value
        else:
            io.mouse_pos = -1, -1

        io.mouse_down[0] = self._mouse_pressed[0] or (mouse_mask & SDL_BUTTON(SDL_BUTTON_LEFT)) != 0
        io.mouse_down[1] = self._mouse_pressed[1] or (mouse_mask & SDL_BUTTON(SDL_BUTTON_RIGHT)) != 0
        io.mouse_down[2] = self._mouse_pressed[2] or (mouse_mask & SDL_BUTTON(SDL_BUTTON_MIDDLE)) != 0
        self._mouse_pressed = [False, False, False]

        io.mouse_wheel = self._mouse_wheel
        self._mouse_wheel = 0


def main():

    window, gl_context = impl_pysdl2_init()
    renderer = FixedPipelineSDL2Renderer(window)
    print(gl.glGetString(gl.GL_VERSION))

    running = True
    event = SDL_Event()
    while running:
        while SDL_PollEvent(ctypes.byref(event)) != 0:
            if event.type == SDL_QUIT:
                running = False
                break
            renderer.process_event(event)
        renderer.process_inputs()

        imgui.new_frame()

        if imgui.begin_main_menu_bar():
            if imgui.begin_menu("File", True):

                clicked_quit, selected_quit = imgui.menu_item(
                    "Quit", 'Cmd+Q', False, True
                )

                if clicked_quit:
                    exit(1)

                imgui.end_menu()
            imgui.end_main_menu_bar()

        imgui.show_test_window()

        imgui.begin("Custom window", True)
        imgui.text("Bar")
        imgui.text_colored("Eggs", 0.2, 1., 0.)
        imgui.end()

        gl.glClearColor(1., 1., 1., 1)
        gl.glClear(gl.GL_COLOR_BUFFER_BIT)

        imgui.render()

        SDL_GL_SwapWindow(window)

    renderer.shutdown()
    SDL_GL_DeleteContext(gl_context)
    SDL_DestroyWindow(window)
    SDL_Quit()


def impl_pysdl2_init():
    width, height = 1280, 720
    window_name = "minimal ImGui/SDL2 example"

    if SDL_Init(SDL_INIT_EVERYTHING) < 0:
        print("Error: SDL could not initialize! SDL Error: " + SDL_GetError())
        exit(1)

    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1)
    SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 24)
    SDL_GL_SetAttribute(SDL_GL_STENCIL_SIZE, 8)
    SDL_GL_SetAttribute(SDL_GL_ACCELERATED_VISUAL, 1)
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLEBUFFERS, 1)
    SDL_GL_SetAttribute(SDL_GL_MULTISAMPLESAMPLES, 16)
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_FLAGS, SDL_GL_CONTEXT_FORWARD_COMPATIBLE_FLAG)
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 2)
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 1)
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_COMPATIBILITY)

    SDL_SetHint(SDL_HINT_MAC_CTRL_CLICK_EMULATE_RIGHT_CLICK, b"1")
    SDL_SetHint(SDL_HINT_VIDEO_HIGHDPI_DISABLED, b"1")

    window = SDL_CreateWindow(window_name.encode('utf-8'),
                              SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
                              width, height,
                              SDL_WINDOW_OPENGL|SDL_WINDOW_RESIZABLE)

    if window is None:
        print("Error: Window could not be created! SDL Error: ", SDL_GetError())
        exit(1)

    gl_context = SDL_GL_CreateContext(window)
    if gl_context is None:
        print("Error: Cannot create OpenGL Context! SDL Error: ", SDL_GetError())
        exit(1)

    # SDL_GL_MakeCurrent(window, gl_context)
    # if SDL_GL_SetSwapInterval(1) < 0:
    #     print("Warning: Unable to set VSync! SDL Error: ", SDL_GetError())
    #     exit(1)

    return window, gl_context


if __name__ == "__main__":
    main()
