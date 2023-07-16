#!/usr/bin/env python3
from __future__ import absolute_import

import sys

import pygame
import OpenGL.GL as gl

from imgui.integrations.pygame import PygameRenderer
import imgui


def draw_menu_strip():
    if imgui.begin_main_menu_bar():
        if imgui.begin_menu("File", True):

            clicked_quit, selected_quit = imgui.menu_item(
                    "Quit", 'Cmd+Q', False, True
                )

            if clicked_quit:
                sys.exit(1)

            imgui.end_menu()
    imgui.end_main_menu_bar()


def check_for_win_close(renderer):
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            sys.exit()

        renderer.process_event(event)


def main():
    # Init `pygame` and declare window `size`
    pygame.init()
    size = 1920, 100

    # Display mode options
    pygame.display.set_mode(size,
                            pygame.DOUBLEBUF |
                            pygame.OPENGL |
                            pygame.RESIZABLE)

    # Create imgui context and get a handle to pygame renderer via `impl`
    imgui.create_context()
    impl = PygameRenderer()

    io = imgui.get_io()
    io.display_size = size

    while 1:
        # Check if user closes program and handle it
        check_for_win_close(impl)

        # Start new frame
        imgui.new_frame()

        # Create menu strip
        draw_menu_strip()

        imgui.begin("Lambda IDE", True)
        imgui.end()

        # Specify clear values for the color buffers
        gl.glClearColor(1, 1, 1, 1)

        # Clear buffers to preset values
        gl.glClear(gl.GL_COLOR_BUFFER_BIT)

        # Finalize frame, set rendering data, and run render callback (if set).
        imgui.render()

        # Draw everything to screen using Pygame `impl` renderer
        impl.render(imgui.get_draw_data())

        # Update the contents of the entire display
        pygame.display.flip()


if __name__ == "__main__":
    main()
