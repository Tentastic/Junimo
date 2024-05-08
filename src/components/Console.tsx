import {listen} from "@tauri-apps/api/event";
import {Download} from "@models/download.ts";
import {useEffect, useState} from "react";
import {clsx} from "clsx";
import {ConsoleModel} from "@models/console.ts";
import {invoke} from "@tauri-apps/api/core";
import {ModInfos} from "@models/mods.ts";

export default function Console({playing, bigConsole}: {playing: boolean, bigConsole: boolean}) {
    const [lines, setLines] = useState<string[]>([]);


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
        };

        let unsubscribeEvent = listen('console', handleNewData);
        greet();

        return () => {
            unsubscribeEvent.then((unsub) => unsub());
        };
    }, []);

    return (
        <div className={clsx(
            "w-full h-full flex overflow-auto",
            (!playing && !bigConsole) && lines.length > 5 || (playing || bigConsole) && lines.length > 21 ? "flex-col-reverse" : "flex-col mt-1"
        )}>
            <div>
                {lines.map((line, index) => (
                    <div key={index} dangerouslySetInnerHTML={{__html: line}}></div>
                ))}
            </div>
        </div>
    )
}