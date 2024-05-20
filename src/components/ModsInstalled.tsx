import {invoke} from "@tauri-apps/api/core";
import {ModInfos} from "@models/mods.ts";
import {useEffect, useState} from "react";
import {clsx} from "clsx";
import {ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger} from "@components/ui/context-menu.tsx";
import {Input} from "@components/ui/input.tsx";
import Wrapper from "@components/ui/wrapper.tsx";
import UninstallMod from "@components/UninstallMod.tsx";
import UninstallAll from "@components/UninstallAll.tsx";
import {useModsState} from "@components/ModsProvider.tsx";
import {useTranslation} from "react-i18next";


export default function ModsInstalled() {
    const { installedMods, selectedAdd } = useModsState();
    const [groupedMods, setGroupedMods] = useState<{ [key: string]: ModInfos[] }>({});
    const { t } = useTranslation('home');

    const groupModsByGroup = (mods: ModInfos[]): { [key: string]: ModInfos[] } => {
        return mods.reduce((groups, mod) => {
            const group = mod.group || 'Ungrouped';
            if (!groups[group]) {
                groups[group] = [];
            }
            groups[group].push(mod);
            return groups;
        }, {} as { [key: string]: ModInfos[] });
    };

    async function getMods() {
        const mods = await invoke<ModInfos[]>('get_installed_mods');
        installedMods[1](mods);
        setGroupedMods(groupModsByGroup(mods));
    }

    function doSearch(search: string) {
        for (let mod of installedMods[0]) {
            mod.invisible = !mod.name.toLowerCase().includes(search.toLowerCase());
        }
        installedMods[1]([...installedMods[0]]);
        setGroupedMods(groupModsByGroup(installedMods[0]));
    }

    function unselectAll() {
        selectedAdd[1]([]);
    }

    function setIndex(modName: string) {
        if (selectedAdd[0].includes(modName)) {
            selectedAdd[1](selectedAdd[0].filter(i => i !== modName));
        }
        else {
            selectedAdd[1]([...selectedAdd[0], modName]);
        }
    }

    function toggleGroup(groupName: string) {
        if (groupedMods[groupName].some(mod => selectedAdd[0].includes(mod.name))) {
            selectedAdd[1](selectedAdd[0].filter(i => !groupedMods[groupName].some(mod => mod.name === i)));
        } else {
            selectedAdd[1]([...selectedAdd[0], ...groupedMods[groupName].map(mod => mod.name)]);
        }
    }

    useEffect(() => {
        getMods();
        selectedAdd[1]([]);

        return () => {
            selectedAdd[1]([]);
        }
    }, []);

    return (
        <div className="relative h-full flex flex-col w-[30vw] border-border pl-3 border rounded-lg">
            <div className="absolute -top-5 bg-background left-2 p-2 px-4">
                <h2 className="text-lg">{t("installedMods")}</h2>
            </div>
            <div className="flex gap-2 mt-6 mx-2 mr-6">
                <div
                    className="w-full transform duration-150 cursor-pointer bg-muted hover:bg-muted-dark rounded-lg flex justify-between">
                    <Input placeholder="Search..." onChange={x => doSearch(x.target.value)}/>
                </div>
            </div>
            <div className="flex flex-col gap-2 w-full aboslute mt-2 overflow-auto relative p-1 pb-4 pr-4">
                {Object.keys(groupedMods).map((group) => (
                    <>
                        {group === 'Ungrouped' ? (
                            <>
                                {groupedMods[group].map((mod, index) => (
                                    <div key={index} className={clsx(
                                        mod.invisible && "hidden"
                                    )}>
                                        <ContextMenu>
                                            <ContextMenuTrigger>
                                                <div onClick={i => setIndex(mod.name)}>
                                                    <Wrapper mod={mod} selected={selectedAdd[0].includes(mod.name)}/>
                                                </div>
                                            </ContextMenuTrigger>
                                            <ContextMenuContent>
                                                <ContextMenuItem onClick={unselectAll}>{t("unselect")}</ContextMenuItem>
                                                <UninstallMod name={mod.name} />
                                                <UninstallAll mods={installedMods[0]} names={selectedAdd[0]} />
                                            </ContextMenuContent>
                                        </ContextMenu>
                                    </div>
                                ))}
                            </>
                        ) : (
                            <ContextMenu>
                                <ContextMenuTrigger>
                                    <div key={group}
                                         className="bg-groups transition duration-150 cursor-pointer hover:bg-groups-dark rounded-lg p-2"
                                         onClick={x => toggleGroup(group)}>
                                        <h2 className="text-lg text-muted-foreground mb-2">{group}</h2>
                                        <div className="flex flex-col gap-2">
                                            {groupedMods[group].map((mod, index) => (
                                                <div key={index} className={clsx(
                                                    mod.invisible && "hidden"
                                                )}>
                                                    <ContextMenu>
                                                        <ContextMenuTrigger>
                                                            <div>
                                                                <Wrapper mod={mod}
                                                                         selected={selectedAdd[0].includes(mod.name)}/>
                                                            </div>
                                                        </ContextMenuTrigger>
                                                        <ContextMenuContent>
                                                            <ContextMenuItem onClick={unselectAll}>{t("unselect")}</ContextMenuItem>
                                                            <UninstallMod name={mod.name}/>
                                                            <UninstallAll mods={installedMods[0]}
                                                                          names={selectedAdd[0]}/>
                                                        </ContextMenuContent>
                                                    </ContextMenu>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                </ContextMenuTrigger>
                                <ContextMenuContent>
                                    <ContextMenuItem onClick={unselectAll}>{t("unselect")}</ContextMenuItem>
                                    <UninstallAll mods={installedMods[0]}
                                                  names={selectedAdd[0]}/>
                                </ContextMenuContent>
                            </ContextMenu>
                        )}
                    </>
                ))}
            </div>
        </div>
    )
}