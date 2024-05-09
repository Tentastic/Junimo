import {invoke} from "@tauri-apps/api/core";
import {Profile} from "@models/profile.ts";
import {Dispatch, SetStateAction, useEffect} from "react";
import Wrapper from "@components/ui/wrapper.tsx";
import {clsx} from "clsx";
import {Input} from "@components/ui/input.tsx";
import {ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger} from "@components/ui/context-menu.tsx";
import UninstallMod from "@components/UninstallMod.tsx";
import UninstallAll from "@components/UninstallAll.tsx";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger, } from "@components/ui/dropdown-menu"

export default function Mods({setKey, profile, setProfile, selected, setSelected, className}:
    {setKey: Dispatch<SetStateAction<number>>, profile: Profile | undefined, setProfile: Dispatch<SetStateAction<Profile | undefined>>, selected: number[], setSelected: Dispatch<SetStateAction<number[]>>, className: string | undefined}) {

    async function loadProfile() {
        const newProfile = await invoke<Profile>('get_current_profile');
        setProfile(newProfile);
    }

    function setIndex(index: number) {
        if (selected.includes(index)) {
            setSelected(selected.filter(i => i !== index));
        }
        else {
            setSelected([...selected, index]);
        }
    }

    function doSearch(search: string) {
        if (profile?.mods !== undefined) {
            for (let mod of profile?.mods) {
                mod.invisible = !mod.name.toLowerCase().includes(search.toLowerCase());
            }

            setProfile({...profile});
        }
    }

    function unselectAll() {
        setSelected([]);
    }

    useEffect(() => {
        loadProfile();
        setSelected([]);
    }, []);

    return (
        <div className={clsx(
            "relative w-full flex flex-col border-border pl-3 border rounded-lg flex-1",
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
                {profile?.mods.map((mod, index) => (
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
                                <UninstallAll mods={profile?.mods} numbers={selected} setKey={setKey}/>
                            </ContextMenuContent>
                        </ContextMenu>
                    </div>
                ))}
            </div>
        </div>
    )
}