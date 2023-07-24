#!/usr/bin/python

import sys, os, subprocess 
from pathlib import Path
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
from PyQt6.QtGui import QIcon, QAction, QFont, QFontMetricsF
from qt_material import apply_stylesheet
from ansi2html import Ansi2HTMLConverter
from pygments import highlight
from pygments.lexers import RustLexer
from pygments.formatters import Terminal256Formatter

class LambdaIde(QMainWindow):
    def __init__(self):
        super(LambdaIde, self).__init__()

        self.sourceFileData = None
        self.sourceFilePath = None
        self.fullyLoaded    = False

        self.initUI()

    
    def updateHighlighting(self):
        if self.fullyLoaded:
            content = self.playground.toPlainText()

            print(content)
            
            self.playground.blockSignals(True)
            # Add syntax highlighting to file 
            ansi_content = highlight(content, 
                                     RustLexer(), 
                                     Terminal256Formatter(style="github-dark"))

            ansi_content = ansi_content.rstrip()

            # Convert ANSI colors to QT html 
            ansiConverter = Ansi2HTMLConverter(latex=False)
            html_content  = ansiConverter.convert(ansi_content, ensure_trailing_newline=False)

            pos = self.playground.textCursor().position()

            # Write loaded source file to playground 
            self.playground.clear()
            self.playground.appendHtml(html_content)
            cursor = self.playground.textCursor()
            cursor.setPosition(min(pos, len(self.playground.toPlainText())))
            self.playground.setTextCursor(cursor)
            self.playground.blockSignals(False)



    def saveFileHandler(self):

        # Make sure we open a source file first before saving
        if self.sourceFilePath == None:
            QMessageBox.critical(self, "Attention", "Make sure to open a file first.")
            return 

        # Make sure we use the current playground data
        self.sourceFileData = self.playground.toPlainText()

        # Save current playground work back to disk
        with open(self.sourceFilePath, "w") as sourceFile:
            sourceFile.write(self.sourceFileData)

    def compileAndRunHandler(self):
        
        # Make sure we open a source file first before running Lambda compiler
        if self.sourceFilePath == None:
            QMessageBox.critical(self, "Attention", "Make sure to open a file first.")
            return 

        # Run Morehead Lambda Compiler on source file and get console output
        compilerProcess = subprocess.Popen(["compiler/debug/mlc", "--source-path", f"{self.sourceFilePath}"], stdout=subprocess.PIPE,
                                            stderr=subprocess.PIPE, text=True)
        compilerOutput, _err = compilerProcess.communicate()

        # Create an ansi code to html converter. 
        # We use this to properly display Lambda's compiler output
        ansiConverter = Ansi2HTMLConverter(latex=False)
        
        # Clean the compiler output to remove ansi color codes
        htmlCompilerOutput = ansiConverter.convert(compilerOutput)

        # Write compiler output to the terminal output text widget
        self.termOutput.appendHtml(htmlCompilerOutput)



    def openFileHandler(self):
        # File dialog box for open Logic
        currentDirectory = os.getcwd()
        self.fileDialog = QFileDialog(
            self, "Open Source File", currentDirectory, "Source Files (*.txt *.la)"
        )

        # Unwrap source file path
        self.sourceFilePath = Path(self.fileDialog.getOpenFileName()[0])
        print(type(self.sourceFilePath))

        if self.sourceFilePath.suffix != ".lm":
            file_name = self.sourceFilePath.name
            QMessageBox.warning(self, 
                                "Alert", 
                                f"Since `{file_name}` does not end with `.lm`, it won't compile!")


        # If user clicks "cancel", just skip opening file
        if self.sourceFilePath != "":
            # Read source file out into the `codePlayground` text box
            with open(self.sourceFilePath, "r") as sourceFile:
                content = sourceFile.read()

                # Add syntax highlighting to file 
                ansi_content = highlight(content, 
                                         RustLexer(stripnl=False, ensurenl=False), 
                                         Terminal256Formatter(style="github-dark"))
        
                # Convert ANSI colors to QT html 
                ansiConverter = Ansi2HTMLConverter(latex=False)
                html_content  = ansiConverter.convert(ansi_content)

                # Keep track of loaded file 
                self.sourceFileData = content
                
                # Write loaded source file to playground 
                self.playground.clear()
                self.playground.appendHtml(html_content)
                self.fullyLoaded = True

    def openInfoHandler(self):
        infoMsg = " Author: Dalton Hensley\n Program: MLC IDE\n" \
        " License: MIT\n\n This program is meant to serve as an interface to " \
        "the Morehead Lambda Compiler. You may write small Lambda programs " \
        "in the `Playground` box and execute programs via the `compile and run` " \
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

        # Logic to save source file via menu strip
        saveAct = QAction("&Save", self)
        saveAct.setShortcut("Ctrl+S")
        saveAct.setStatusTip("Save file")
        saveAct.triggered.connect(self.saveFileHandler)

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
        fileMenu.addAction(saveAct)
        fileMenu.addAction(exitAct)

        # Options under the "Help" tab in menubar
        fileMenu = menubar.addMenu("&Help")
        fileMenu.addAction(infoAct)

        # Add a button to run compiler on file and run program
        compileAndRunBtn = QPushButton("Compile and Run")
        compileAndRunBtn.clicked.connect(self.compileAndRunHandler)
        compileAndRunBtn.setMaximumWidth(160)

        # Add helpful status bar in bottom left corner
        statusBar = QStatusBar(self)
        self.setStatusBar(statusBar)

        # Add input text box for writing code
        self.playground = QPlainTextEdit("")
        self.playground.setTabStopDistance(
        QFontMetricsF(self.playground.font()).horizontalAdvance(' ') * 4)
        self.playground.textChanged.connect(self.updateHighlighting)

        # Add output text box for terminal output
        self.termOutput = QPlainTextEdit("")
        self.termOutput.setMaximumHeight(400)
        self.termOutput.setEnabled(True)

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

    # Initalize PyQt6
    app = QApplication(sys.argv)

    # setup stylesheet
    apply_stylesheet(app, theme='dark_blue.xml')


    # Initalize IDE and display it to screen
    ide = LambdaIde()
    ide.show()

    # Return exit code based on the exit code of the IDE
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
