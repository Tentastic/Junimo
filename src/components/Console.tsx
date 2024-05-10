import {listen} from "@tauri-apps/api/event";
import {Download} from "@models/download.ts";
import {useEffect, useRef, useState} from "react";
import {clsx} from "clsx";
import {ConsoleModel} from "@models/console.ts";
import {invoke} from "@tauri-apps/api/core";
import {ModInfos} from "@models/mods.ts";

export default function Console({playing, bigConsole}: {playing: boolean, bigConsole: boolean}) {
    const [lines, setLines] = useState<string[]>([]);

    const [size, setSize] = useState({ width: 200, height: 128 });
    const [isResizing, setIsResizing] = useState(false);
    const [startPos, setStartPos] = useState({ x: 0, y: 0 });
    const [flexReverse, setFlexReverse] = useState(false);
    const containerRef = useRef(null);

    function checkReverse(newHeight: number) {
        const containerCheck = containerRef.current;
        if (!containerCheck) return;
        const container = containerCheck as HTMLElement;
        const isOverflowing = container.scrollHeight > newHeight;
        setFlexReverse(isOverflowing);
    }

    const onMouseDown = (e: React.MouseEvent) => {
        setIsResizing(true);
        setStartPos({
            x: e.clientX,
            y: e.clientY,
        });

        const html = document.documentElement;
        html.classList.add("select-none");
    };

    const onMouseMove = (e: MouseEvent) => {
        if (isResizing) {
            const newWidth = size.width + e.clientX - startPos.x;
            let newHeight = size.height + startPos.y - e.clientY;

            const sev = document.documentElement.clientHeight * 0.5;

            if (newHeight < 128)
                newHeight = 128;
            else if (newHeight > sev)
                newHeight = sev;

            setSize({ width: newWidth, height: newHeight });
            setStartPos({ x: e.clientX, y: e.clientY });

            checkReverse(newHeight);
        }
    };

    const onMouseUp = () => {
        setIsResizing(false);
        const html = document.documentElement;
        html.classList.remove("select-none");
    };


    function addLine(line: ConsoleModel) {
        setLines(prevLines => {
            const newLines = [...prevLines];

            if (newLines.length > 10000) {
                newLines.shift();
            }

            newLines.push(line.content);

            return newLines;
        });
    }

    function modifyLine(line: ConsoleModel) {
        setLines(prevLines => {
            const newLines = [...prevLines];

            if (newLines.length > 0) {
                newLines[newLines.length - 1] = line.content;
            }

            return newLines;
        });
    }

    async function greet() {
        const result = await invoke("greet");
        const data = [result as string];
        setLines(data);
    }



    useEffect(() => {
        const handleNewData = (event: any) => {
            const data = event.payload as ConsoleModel;
            if (data.mode === 0) {
                addLine(data);
            }
            else if (data.mode === 1) {
                modifyLine(data);
            }
            else if (data.mode === 2) {
                addLine(data);
            }
            checkReverse(size.height);
        };

        let unsubscribeEvent = listen('console', handleNewData);
        greet();

        return () => {
            unsubscribeEvent.then((unsub) => unsub());
        };
    }, []);

    useEffect(() => {
        if (isResizing) {
            window.addEventListener('mousemove', onMouseMove);
            window.addEventListener('mouseup', onMouseUp);
        } else {
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);
        }

        // Cleanup function to remove event listeners
        return () => {
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);
        };
    }, [isResizing]);

    return (
        <div style={{ height: `${size.height}px` }}
            className={clsx(
            "h-32 min-h-32 max-h-[50vh] border-border bg-card hover:bg-muted transition duration-150 cursor-pointer pl-2 col-span-4 pb-2 border rounded-lg w-full overflow-hidden",
        )}
            >
            <div className="w-full h-2 min-h-2 max-h-2 cursor-n-resize"
                 onMouseDown={onMouseDown} />
            <div ref={containerRef} className={clsx(
                "h-full flex overflow-auto",
                flexReverse ? "flex-col-reverse" : "flex-col",
            )}>
                <div>
                    {lines.map((line, index) => (
                        <div key={index} dangerouslySetInnerHTML={{__html: line}}></div>
                    ))}
                </div>
            </div>
        </div>
    )
}