#!/usr/bin/python

import sys
from PyQt6.QtWidgets import (
    QMainWindow,
    QApplication,
    QFileDialog,
    QPlainTextEdit,
    QVBoxLayout,
    QWidget,
    QMenuBar,
    QStatusBar,
    QPushButton,
    QMessageBox,
)
from PyQt6.QtGui import QIcon, QAction

# class OpenFileAction(QAction):
# def __init__(self):
# super().__init__()

# self.initAction()


# def initAction(self):
# self = QAction
# self.text = "&Open"


class LambdaIde(QMainWindow):
    def __init__(self):
        super(LambdaIde, self).__init__()

        self.initUI()

    def openFileHandler(self):
        # File dialog box for open Logic
        self.fileDialog = QFileDialog(
            self, "Open Source File", "/home/", "Source Files (*.txt *.la)"
        )

        # Unwrap source file path
        sourceFilePath = self.fileDialog.getOpenFileName()[0]

        # If user clicks "cancel", just skip opening file
        if sourceFilePath != "":
            # Read source file out into the `codePlayground` text box
            with open(sourceFilePath, "r") as sourceFile:
                self.playground.clear()
                self.playground.insertPlainText(str(sourceFile.read()))

    def openInfoHandler(self):
        infoMsg = " Author: Dalton Hensley\n Program: MLC IDE\n" \
        "License: MIT\n\n This program is meant to serve as an interface to" \
        "the Morehead Lambda Compiler. You may write small Lambda programs" \
        "in the `Playground` box and execute programs via the `compile and run`" \
        "button.\n"

        QMessageBox.information(self, "Info", f"{infoMsg}")

    def initUI(self):
        # Vertical window layout
        layout = QVBoxLayout()
        layout.setSpacing(10)

        # Logic to open source file via menu strip
        openAct = QAction(QIcon("file.png"), "&Open", self)
        openAct.setShortcut("Ctrl+O")
        openAct.setStatusTip("Open file")
        openAct.triggered.connect(self.openFileHandler)

        # Logic to exit IDE via menu strip
        exitAct = QAction(QIcon("exit.png"), "&Quit", self)
        exitAct.setShortcut("Ctrl+Q")
        exitAct.setStatusTip("Exit application")
        exitAct.triggered.connect(QApplication.instance().quit)

        # Logic to open the Info popup
        infoAct = QAction("&Info", self)
        infoAct.setShortcut("Ctrl+I")
        infoAct.setStatusTip("About Program")
        infoAct.triggered.connect(self.openInfoHandler)

        # Create menu bar
        menubar = QMenuBar()

        # Options under the "File" tab in menubar
        fileMenu = menubar.addMenu("&File")
        fileMenu.addAction(openAct)
        fileMenu.addAction(exitAct)

        # Options under the "Help" tab in menubar
        fileMenu = menubar.addMenu("&Help")
        fileMenu.addAction(infoAct)

        # Add a button to run compiler on file and run program
        compileAndRunBtn = QPushButton("Compile and Run")
        compileAndRunBtn.setMaximumWidth(130)

        # Add helpful status bar in bottom left corner
        statusBar = QStatusBar(self)
        self.setStatusBar(statusBar)

        # Add input text box for writing code
        self.playground = QPlainTextEdit("")

        # Add output text box for terminal output
        self.termOutput = QPlainTextEdit("")
        self.termOutput.setEnabled(False)

        # Add above widgets to the vertical layout
        layout.addWidget(menubar)
        layout.addWidget(compileAndRunBtn)
        layout.addWidget(self.playground)
        layout.addWidget(self.termOutput)

        ideWidget = QWidget()
        ideWidget.setLayout(layout)

        self.setGeometry(300, 300, 350, 250)
        self.setWindowTitle("Simple menu")
        self.setCentralWidget(ideWidget)


def main():
    app = QApplication(sys.argv)
    ide = LambdaIde()
    ide.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
