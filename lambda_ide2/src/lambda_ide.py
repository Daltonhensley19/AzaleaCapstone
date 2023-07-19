import dearpygui.dearpygui as dpg
import dearpygui.demo as demo

from strip_ansi import strip_ansi
import sys
import subprocess
from pathlib import Path
import enum 


def print_me(sender):
    print(f"Menu Item: {sender}")


def exit_handler():
    dpg.stop_dearpygui()
   

help_msg = """\t\t\t  Author: Dalton Hensley
              Program: MLC IDE
              License: MIT 

              This program is meant to serve as an interface 
              to the Morehead Lambda Compiler.
              You may write small Lambda programs 
              in the `Playground` box and execute 
              programs via the `build and run` button."""


def help_handler():
    with dpg.window(label="Help Window", tag="helpMenu", no_resize=True, no_close=True):
        dpg.add_text(help_msg)
        dpg.add_separator()
        dpg.add_button(label="Ok", callback=lambda: dpg.delete_item("helpMenu"),show=True)


def handle_file_dialog_ok(sender, app_data, user_data):
    
    # Unwrap source file path
    path = list(app_data["selections"].values())[0]
    
    # Compile and run button now stores the path
    dpg.set_item_user_data("compileRunBtn", path)

    # Write source file as string into the "Playground"
    with open(path, "r") as source_file:
        source_file_data = source_file.read()

        # Workaround: we must remove the Playground and redraw with loaded 
        # source file text.
        dpg.delete_item("playgroundTBox")
        dpg.add_input_text(parent="playgroundWin",  show=True, before="termOutWin", multiline=True, height=600, width=500, tag="playgroundTBox")
        dpg.set_value("playgroundTBox", str(source_file_data))


def handle_file_dialog_cancel(sender, app_data, user_data):
    print("Sender: ", sender)
    print("App Data: ", app_data)


def with_file_dialog():
    with dpg.file_dialog(
        directory_selector=False,
        show=False,
        callback=handle_file_dialog_ok,
        cancel_callback=handle_file_dialog_cancel,
        id="file_dialog_id",
        width=700,
        height=400,
    ):
        dpg.add_file_extension(".*")
        dpg.add_file_extension(
            "Source files (*.sp){.sp}", color=(0, 255, 255, 255)
        )


def create_menu_strip():
    with dpg.viewport_menu_bar():
        with_file_dialog()
        with dpg.menu(label="File"):
            dpg.add_menu_item(label="Open", callback=lambda: dpg.show_item("file_dialog_id"))
            dpg.add_menu_item(label="Save", callback=print_me)
            dpg.add_menu_item(label="Save As", callback=print_me)
            dpg.add_menu_item(label="Exit", callback=exit_handler)

        with dpg.menu(label="Info"):
            dpg.add_menu_item(label="Help", callback=help_handler)


# Handler to compile and run Lambda program 
def compile_and_run_handler():
    # Check to make sure that the source file path is set first
    if dpg.get_item_user_data("compileRunBtn") == None:
        print("make sure to set source path")
    else:
        source_path = dpg.get_item_user_data("compileRunBtn")
        compiler_process = subprocess.Popen(["compiler/debug/mlc", "--source-path", f"{source_path}"], stdout=subprocess.PIPE,
                                            stderr=subprocess.PIPE, text=True)
        compiler_output, _err = compiler_process.communicate()

        # Clean the compiler output to remove ansi color codes
        cleaned_compiler_output = strip_ansi(compiler_output)

        # Workaround: we must remove the Playground and redraw with loaded 
        # source file text.
        dpg.delete_item("termOutTBox")
        dpg.add_text(parent="termOutWin",  show=True, tag="termOutTBox", wrap=500)
        dpg.set_value("termOutTBox", str(cleaned_compiler_output))
    


def run_ide():
    dpg.create_context()

    def callback(sender, app_data):
        print("OK was clicked.")
        print("Sender: ", sender)
        print("App Data: ", app_data)

    def cancel_callback(sender, app_data):
        print("Cancel was clicked.")
        print("Sender: ", sender)
        print("App Data: ", app_data)
    
    
    with dpg.font_registry():
        with dpg.font("./assets/TTF/SourceCodePro-Regular.ttf", 16, default_font=True):
            dpg.add_font_range(0x2580, 0x259F)
            dpg.add_font_range(0x2500, 0x257F)
    
    with dpg.window(label="Lambda IDE", width=800, height=800, tag = "primaryWin"):
        create_menu_strip()
        with dpg.child_window(label="Playground", width=800, height=800, tag = "playgroundWin"):
            with dpg.group(horizontal=True):
                dpg.add_button(label="Compile and Run", callback=compile_and_run_handler, tag="compileRunBtn")
                dpg.add_input_text(show=True, multiline=True, height=600, width=500, tag="playgroundTBox")
            with dpg.child_window(label="Terminal Output", width=800, height=400, tag = "termOutWin"):
                dpg.add_text(show=True, tag="termOutTBox", wrap=500)

    dpg.show_font_manager()
    demo.show_demo()

    manage_dpg()


def manage_dpg():
    dpg.create_viewport(title="Lambda IDE")
    dpg.setup_dearpygui()
    dpg.show_viewport()
    dpg.set_primary_window("primaryWin", True)
    dpg.start_dearpygui()
    dpg.destroy_context()


