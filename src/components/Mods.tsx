import {invoke} from "@tauri-apps/api/tauri";
import {Config as ConfigModel} from "@models/config.ts";
import {Profile} from "@models/profile.ts";
import {Dispatch, MutableRefObject, SetStateAction, useEffect, useImperativeHandle, useState} from "react";
import Wrapper from "@components/ui/wrapper.tsx";
import {clsx} from "clsx";


export default function Mods({profile, setProfile, selected, setSelected, className}:
    {profile: Profile | undefined, setProfile: Dispatch<SetStateAction<Profile | undefined>>, selected: number[], setSelected: Dispatch<SetStateAction<number[]>>, className: string | undefined}) {

    async function loadProfile() {
        const profiles = await invoke<Profile[]>('get_profile');
        for (let profile of profiles) {
            if (profile.currently) {
                setProfile(profile);
                break;
            }
        }
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
        loadProfile();
    }, []);

    return (
        <div className={clsx(
            "flex flex-col gap-2 w-full h-full mt-6 overflow-auto pr-4",
            className
        )}>
            {profile?.mods.map((mod, index) => (
                <div key={index} onClick={i => setIndex(index)}>
                    <Wrapper mod={mod} selected={selected.includes(index)}   />
                </div>
            ))}
            {profile?.mods.map((mod, index) => (
                <div key={index} onClick={i => setIndex(index)}>
                    <Wrapper mod={mod} selected={selected.includes(index)}   />
                </div>
            ))}
        </div>
    )
}