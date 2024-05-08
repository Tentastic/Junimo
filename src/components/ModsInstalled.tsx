import {invoke} from "@tauri-apps/api/tauri";
import {ModInfos} from "@models/mods.ts";
import {Dispatch, SetStateAction, useEffect} from "react";
import {clsx} from "clsx";
import {ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger} from "@components/ui/context-menu.tsx";
import {Input} from "@components/ui/input.tsx";
import Wrapper from "@components/ui/wrapper.tsx";
import {TabsContent} from "@components/ui/tabs.tsx";
import UninstallMod from "@components/UninstallMod.tsx";
import UninstallAll from "@components/UninstallAll.tsx";


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
        <TabsContent value="mods" className="flex-1 mt-7">
            <div className="relative flex-grow h-full p-4 pr-0 px-3 pt-0 border-border border rounded-lg">
                <div className="absolute -top-5 left-2 bg-background p-2 px-4">
                    <h2 className="text-lg">Mods Installed</h2>
                </div>
                <div className={clsx(
                    "flex flex-col gap-2 w-full h-full mt-6 pr-4 overflow-y-auto",
                    className
                )}>
                    <div className="w-full transform duration-150 cursor-pointer bg-muted hover:bg-muted-dark rounded-lg flex justify-between">
                        <Input placeholder="Search..." onChange={x => doSearch(x.target.value)}/>
                    </div>
                    {modList.map((mod, index) => (
                        <ContextMenu>
                            <ContextMenuTrigger>
                                <div key={index} onClick={i => setIndex(index)}>
                                    <Wrapper mod={mod} selected={selected.includes(index)}/>
                                </div>
                            </ContextMenuTrigger>
                            <ContextMenuContent>
                                <ContextMenuItem onClick={unselectAll}>Unselect all</ContextMenuItem>
                                <UninstallMod name={mod.name} setKey={setKey} />
                                <UninstallAll mods={modList} numbers={selected} setKey={setKey} />
                            </ContextMenuContent>
                        </ContextMenu>
                    ))}
                </div>
            </div>
        </TabsContent>
    )
}