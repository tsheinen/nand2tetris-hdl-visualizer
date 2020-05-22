## nand2tetris-hdl-visualizer

Utility to visualize HDL files from Nand2Tetris as GraphViz DOT files. 

### Examples

```
CHIP And {
    IN a, b;
    OUT out;

    PARTS:
	Not(in=a, out=nota);
	Not(in=b, out=notb);
	Nor(a=nota, b=notb, out=out);
}
```

is converted to
```DOT
digraph {
	label="And";
	labelloc=top;
	labeljust=left;	Input_4294967295 [label="Input"];
	Output_4294967295 [label="Output"];
	Not_0 [ label=" Not" ];
	Not_1 [ label=" Not" ];
	Nor_2 [ label=" Nor" ];
	Input_4294967295 -> Not_0 [ label=" a" ];
	Input_4294967295 -> Not_1 [ label=" b" ];
	Not_0 -> Nor_2 [ label=" nota" ];
	Not_1 -> Nor_2 [ label=" notb" ];
	Nor_2 -> Output_4294967295 [ label=" out" ];
}
```
which can be rendered using GraphViz as

![And graph](assets/And.png)