import {listen} from "@tauri-apps/api/event";
import {Download} from "@models/download.ts";
import {useEffect, useState} from "react";

export default function Console() {
    const [lines, setLines] = useState<string[]>([]);

    function addLine(line: string) {
        setLines(prevLines => {
            const newLines = [...prevLines];

            if (newLines.length > 10000) {
                newLines.shift();
            }

            newLines.push(line);

            return newLines;
        });
    }

    useEffect(() => {
        const handleNewData = (event: any) => {
            addLine(event.payload);
        };

        let unsubscribeEvent = listen('console', handleNewData);

        return () => {
            unsubscribeEvent.then((unsub) => unsub());
        };
    }, []);

    return (
        <div className="w-full h-full flex flex-col-reverse overflow-auto">
            <div className="mt-auto">
                {lines.map((line, index) => (
                    <div key={index} dangerouslySetInnerHTML={{__html: line}}></div>
                ))}
            </div>
        </div>
    )
}