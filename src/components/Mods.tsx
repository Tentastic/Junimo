import {invoke} from "@tauri-apps/api/core";
import {Profile} from "@models/profile.ts";
import {Dispatch, SetStateAction, useEffect, useState} from "react";
import Wrapper from "@components/ui/wrapper.tsx";
import {clsx} from "clsx";
import {Input} from "@components/ui/input.tsx";
import {ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger} from "@components/ui/context-menu.tsx";
import UninstallMod from "@components/UninstallMod.tsx";
import UninstallAll from "@components/UninstallAll.tsx";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger, } from "@components/ui/dropdown-menu"
import {useModsState} from "@components/ModsProvider.tsx";
import {ModInfos} from "@models/mods.ts";
import {useTranslation} from "react-i18next";

export default function Mods() {
    const { profile, selectedRemove } = useModsState();
    const [groupedMods, setGroupedMods] = useState<{ [key: string]: ModInfos[] }>({});
    const { t } = useTranslation('home');

    const groupModsByGroup = (mods: ModInfos[] | undefined): { [p: string]: ModInfos[] } => {
        if (mods === undefined) return {};
        return mods.reduce((groups, mod) => {
            const group = mod.group || 'Ungrouped';
            if (!groups[group]) {
                groups[group] = [];
            }
            groups[group].push(mod);
            return groups;
        }, {} as { [key: string]: ModInfos[] });
    };

    async function loadProfile() {
        const profilePath = await invoke<string>('profile_path');
        const loadedMods = await invoke<Profile>('get_current_profile', {path: profilePath});

        console.log(loadedMods);
        profile[1](loadedMods);
        setGroupedMods(groupModsByGroup(loadedMods.mods));
        //profile[1](newProfile);
    }

    function setIndex(modNames: string) {
        if (selectedRemove[0].includes(modNames)) {
            selectedRemove[1](selectedRemove[0].filter(i => i !== modNames));
        }
        else {
            selectedRemove[1]([...selectedRemove[0], modNames]);
        }
    }

    function doSearch(search: string) {
        if (profile[0] === undefined) return;
        let searchedMods = profile[0].mods;
        for (let mod of searchedMods) {
            mod.invisible = !mod.name.toLowerCase().includes(search.toLowerCase());
        }
        let newProfile = profile[0];
        newProfile.mods = searchedMods;
        profile[1](newProfile);
        setGroupedMods(groupModsByGroup(searchedMods));
    }

    function unselectAll() {
        selectedRemove[1]([]);
    }

    function toggleGroup(groupName: string) {
        if (groupedMods[groupName].some(mod => selectedRemove[0].includes(mod.name))) {
            selectedRemove[1](selectedRemove[0].filter(i => !groupedMods[groupName].some(mod => mod.name === i)));
        } else {
            selectedRemove[1]([...selectedRemove[0], ...groupedMods[groupName].map(mod => mod.name)]);
        }
    }

    useEffect(() => {
        loadProfile();
        selectedRemove[1]([]);
    }, []);

    return (
        <div className="relative w-full flex flex-col border-border pl-3 border rounded-lg flex-1">
            <div className="absolute -top-5 bg-background left-2 p-2 px-4">
                <h2 className="text-lg">{t("currentMods")}</h2>
            </div>
            <div className="flex gap-2 mt-6 mx-2 mr-6">
                <div
                    className="w-full transform duration-150 cursor-pointer bg-muted hover:bg-muted-dark rounded-lg flex justify-between">
                    <Input placeholder={t("search")} onChange={x => doSearch(x.target.value)}/>
                </div>
            </div>
            <div className="flex flex-col gap-2 w-full mt-2 overflow-auto p-1 pb-4 pr-4">
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
                                                    <Wrapper mod={mod} selected={selectedRemove[0].includes(mod.name)}/>
                                                </div>
                                            </ContextMenuTrigger>
                                            <ContextMenuContent>
                                                <ContextMenuItem onClick={unselectAll}>{t("unselect")}</ContextMenuItem>
                                                <UninstallMod name={mod.name} />
                                                <UninstallAll mods={profile[0]?.mods} names={selectedRemove[0]} />
                                            </ContextMenuContent>
                                        </ContextMenu>
                                    </div>
                                ))}
                            </>
                        ) : (
                            <ContextMenu>
                                <ContextMenuTrigger>
                                    <div key={group}
                                         className={clsx(
                                             "bg-groups transition duration-150 cursor-pointer hover:bg-groups-dark rounded-lg p-2",
                                             groupedMods[group].filter(mod => mod.invisible).length === groupedMods[group].length && "hidden"
                                         )}
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
                                                                         selected={selectedRemove[0].includes(mod.name)}/>
                                                            </div>
                                                        </ContextMenuTrigger>
                                                        <ContextMenuContent>
                                                            <ContextMenuItem onClick={unselectAll}>{t('unselect')}</ContextMenuItem>
                                                            <UninstallMod name={mod.name}/>
                                                            <UninstallAll mods={profile[0]?.mods}
                                                                          names={selectedRemove[0]}/>
                                                        </ContextMenuContent>
                                                    </ContextMenu>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                </ContextMenuTrigger>
                                <ContextMenuContent>
                                    <ContextMenuItem onClick={unselectAll}>{t('unselect')}</ContextMenuItem>
                                    <UninstallAll mods={profile[0]?.mods}
                                                  names={selectedRemove[0]}/>
                                </ContextMenuContent>
                            </ContextMenu>
                        )}
                    </>
                ))}
            </div>
        </div>
    )
}