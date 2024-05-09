import {invoke} from "@tauri-apps/api/core";
import {ModInfos} from "@models/mods.ts";
import {Dispatch, SetStateAction, useEffect} from "react";
import {clsx} from "clsx";
import {ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger} from "@components/ui/context-menu.tsx";
import {Input} from "@components/ui/input.tsx";
import Wrapper from "@components/ui/wrapper.tsx";
import {TabsContent} from "@components/ui/tabs.tsx";
import UninstallMod from "@components/UninstallMod.tsx";
import UninstallAll from "@components/UninstallAll.tsx";
import {
    DropdownMenu,
    DropdownMenuContent, DropdownMenuItem,
    DropdownMenuLabel, DropdownMenuSeparator,
    DropdownMenuTrigger
} from "@components/ui/dropdown-menu.tsx";


export default function ModsInstalled({setKey, modList, setModList, selected, setSelected, className}:
    {setKey: Dispatch<SetStateAction<number>>, modList: ModInfos[], setModList: Dispatch<SetStateAction<ModInfos[]>>, selected: number[], setSelected: Dispatch<SetStateAction<number[]>>, className: string}) {

    async function getMods() {
        const mods = await invoke<ModInfos[]>('get_installed_mods');
        setModList(mods);
    }

    function doSearch(search: string) {
        for (let mod of modList) {
            mod.invisible = !mod.name.toLowerCase().includes(search.toLowerCase());
        }

        setModList([...modList]);
    }

    function unselectAll() {
        setSelected([]);
    }

    function setIndex(index: number) {
        if (selected.includes(index)) {
            setSelected(selected.filter(i => i !== index));
        }
        else {
            setSelected([...selected, index]);
        }
    }

    useEffect(() => {
        getMods();
        setSelected([]);

        return () => {
            setSelected([]);
        }
    }, []);

    return (
        <TabsContent value="mods" className={clsx(
            "relative w-full flex flex-col border-border pl-3 border rounded-lg overflow-auto",
            className
        )}>
            <div className="absolute -top-5 bg-background left-2 p-2 px-4">
                <h2 className="text-lg">Current Mods</h2>
            </div>
            <div className="flex gap-2 mt-6 mx-2 mr-6">
                <div
                    className="w-full transform duration-150 cursor-pointer bg-muted hover:bg-muted-dark rounded-lg flex justify-between">
                    <Input placeholder="Search..." onChange={x => doSearch(x.target.value)}/>
                </div>
                <DropdownMenu>
                    <DropdownMenuTrigger>
                        <button
                            className="w-12 h-full rounded transition duration-150 bg-muted hover:bg-muted-dark flex items-center justify-center">
                            Sort
                        </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                        <DropdownMenuLabel>My Account</DropdownMenuLabel>
                        <DropdownMenuSeparator/>
                        <DropdownMenuItem>Profile</DropdownMenuItem>
                        <DropdownMenuItem>Billing</DropdownMenuItem>
                        <DropdownMenuItem>Team</DropdownMenuItem>
                        <DropdownMenuItem>Subscription</DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>
            <div className={clsx(
                "flex flex-col gap-2 w-full mt-2 overflow-auto p-1 pr-4"
            )}>
                {modList.map((mod, index) => (
                    <div key={index} className={clsx(
                        mod.invisible && "hidden"
                    )}>
                        <ContextMenu>
                            <ContextMenuTrigger>
                                <div onClick={i => setIndex(index)}>
                                    <Wrapper mod={mod} selected={selected.includes(index)}/>
                                </div>
                            </ContextMenuTrigger>
                            <ContextMenuContent>
                                <ContextMenuItem onClick={unselectAll}>Unselect all</ContextMenuItem>
                                <UninstallMod name={mod.name} setKey={setKey}/>
                                <UninstallAll mods={modList} numbers={selected} setKey={setKey}/>
                            </ContextMenuContent>
                        </ContextMenu>
                    </div>
                ))}
            </div>
        </TabsContent>
    )
}