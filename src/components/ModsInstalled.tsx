import {invoke} from "@tauri-apps/api/tauri";
import {ModInfos} from "@models/mods.ts";
import {Dispatch, SetStateAction, useEffect, useState} from "react";
import {appDataDir, appLocalDataDir} from "@tauri-apps/api/path";
import {clsx} from "clsx";
import {ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger} from "@components/ui/context-menu.tsx";
import {Progress} from "@components/ui/progress.tsx";
import {Input} from "@components/ui/input.tsx";
import Wrapper from "@components/ui/wrapper.tsx";


export default function ModsInstalled({modList, setModList, selected, setSelected, className}:
    {modList: ModInfos[], setModList: Dispatch<SetStateAction<ModInfos[]>>, selected: number[], setSelected: Dispatch<SetStateAction<number[]>>, className: string}) {

    async function getMods() {
        const mods = await invoke<ModInfos[]>('get_mods');
        setModList(mods);
    }

    function doSearch(search: string) {
        for (let mod of modList) {
            mod.invisible = !mod.name.toLowerCase().includes(search.toLowerCase());
        }

        setModList([...modList]);
    }

    useEffect(() => {
        getMods();

        return () => {
            setSelected([]);
        }
    }, []);

    function setIndex(index: number) {
        if (selected.includes(index)) {
            setSelected(selected.filter(i => i !== index));
        }
        else {
            setSelected([...selected, index]);
        }
    }

    return (
        <div className={clsx(
            "flex flex-col gap-2 w-full h-full mt-6 pr-4",
            className
        )}>
            <div
                 className={clsx(
                     "w-full transform duration-150 cursor-pointer bg-muted hover:bg-muted-dark rounded-lg flex justify-between"
                 )}>
                <Input placeholder="Search..." onChange={x => doSearch(x.target.value)} />
            </div>
            {modList.map((mod, index) => (
                <ContextMenu>
                    <ContextMenuTrigger>
                        <div key={index} onClick={i => setIndex(index)}>
                            <Wrapper mod={mod} selected={selected.includes(index)}/>
                        </div>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                        <ContextMenuItem>Activate</ContextMenuItem>
                        <ContextMenuItem>Uninstall</ContextMenuItem>
                    </ContextMenuContent>
                </ContextMenu>
            ))}
        </div>
    )
}