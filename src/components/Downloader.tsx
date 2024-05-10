import {listen} from "@tauri-apps/api/event";
import {Dispatch, SetStateAction, useEffect, useState} from "react";
import {Download} from "@models/download.ts";
import {invoke} from "@tauri-apps/api/core";
import {User} from "@models/user.ts";
import {ModInfos} from "@models/mods.ts";
import {clsx} from "clsx";
import {Progress} from "@components/ui/progress.tsx";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
} from "@components/ui/context-menu"
import {TabsContent} from "@components/ui/tabs.tsx";

export default function Downloader({downloadList}: {downloadList: Download[]}) {
    async function stopDownload() {
        await invoke('stop_download');
    }

    function bytesToString(bytes: number) {
        if (bytes / 1000000 < 1) {
            const megabytes = bytes / 1000;
            return `${megabytes.toFixed(2)} KB`;
        }
        else {
            const megabytes = bytes / 1000000;
            return `${megabytes.toFixed(2)} MB`;
        }
    }

    return (
        <div className="relative h-full flex flex-col w-[30vw] border-border pl-3 border rounded-lg">
            <div className="absolute -top-5 bg-background left-2 p-2 px-4">
                <h2 className="text-lg">Downloads</h2>
            </div>
            <div className="flex flex-col gap-2 w-full aboslute mt-2 overflow-auto relative p-1 pb-4 pr-4">
                {downloadList.map((mod, index) => (
                        <ContextMenu>
                            <ContextMenuTrigger>
                                <div key={index}
                                     className="w-full transform duration-150 cursor-pointer bg-muted hover:bg-muted-dark rounded-lg pb-0 overflow-hidden">
                                    <div className="w-full flex justify-between p-2 pb-1">
                                        <p>{mod.name}</p>
                                        <p className="text-zinc-500 h-2">{bytesToString(mod.current)} / {bytesToString(mod.size)}</p>
                                    </div>
                                    <Progress value={mod.current / mod.size * 100}/>
                                </div>
                            </ContextMenuTrigger>
                            <ContextMenuContent>
                                {!mod.aborted && (
                                    <ContextMenuItem onClick={stopDownload}>Cancel</ContextMenuItem>
                                )}
                            </ContextMenuContent>
                        </ContextMenu>
                    )
                )}
            </div>
        </div>
    )
}