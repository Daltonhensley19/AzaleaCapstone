import graphviz as gviz
import matplotlib.pyplot as plt
import json


ast_dict = None
ast_str  = None
with open("../ast_dump/ast.json", "r") as ast_json:
    ast_dict = json.load(ast_json)
    ast_str  = json.dumps(ast_dict, indent=2)


G2 = gviz.Digraph(strict=True)


def walk_list(key, lis):
    for i in lis:
        if isinstance(i, dict):
            walk_dict(key, i)
        elif isinstance(i, str):
            G2.edge(key,i)
        elif i == None:
            G2.node(key, "null")
        else:
            walk_list(key, i)


def walk_dict(key, diction):
        for j, value in diction.items():
            if isinstance(value, dict):
                G2.node(key, color="lightblue", style="filled")
                G2.node(j, color="lightblue", style="filled")
                G2.edge(key, j)
                walk_dict(j, value)
            elif isinstance(value, str):
                G2.node(key, color="lightblue", style="filled")
                G2.node(value, color="maroon", style="filled")
                print(key, j)
                print(j, value)
                if str(j) == "raw_content":
                    G2.edge(key, value)
                else:
                    G2.node(j, color="lightgreen", style="filled")
                    G2.edge(key, j)
                    G2.edge(j, value)


            elif isinstance(value, list):
                print(key)
                G2.edge(key, j)
                walk_list(j, value)
            elif value == None:
                G2.edge(key, j)
                G2.node(j, color="grey", style="filled")


def walk(node):
    for key, item in node.items():
        if isinstance(item, list):
            walk_list(key, item)
        if isinstance(item, dict):
            walk_dict(key, item)
        elif isinstance(item,str):
            G2.node(key,item)
        # elif item == None:
            # G2.edge(key, "null")
    
    return G2

ast_graph   = walk(ast_dict)
print(ast_graph)
ast_graph.render(directory="../ast_dump/")

