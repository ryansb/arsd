<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { sendNotification } from '@tauri-apps/plugin-notification';
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-shell';
import { Confirmation, Partition, useSessionStore } from "../store";
import CountDown from "./CountDown.vue";
import { onMounted } from "vue";

const store = useSessionStore();
const partitions = ref<Partition[]>([]);
const checkToken = ref();
const snackbar = ref(false);
const snackbarMessage = ref("Authentication error");

async function getPartitions() {
    partitions.value = await invoke("get_partitions", {});
    if (partitions.value.length === 0) {
        return
    }
    const s = partitions.value.map((p) => p.slug).reduce((p, _) => p)
    if (s === undefined) return
    store.partitions[s] = { slug: s }
}

await getPartitions();

const unListen = await listen('authorize_device', async (event: any) => {
    console.log("authorize_device event received", event.payload)
})

onMounted(async () => {
    partitions.value.map((p) => {
        tryAuth(p.slug)
    })
});

onBeforeUnmount(() => {
    unListen();
    clearInterval(checkToken.value);
    checkToken.value = null;
});

async function tryAuth(partition: string) {
    let payload: object;
    try {
        console.log("Sending authorize_device")
        payload = await invoke("authorize_device", { authEvent: { partition_name: partition } });
        console.log("received authorize_device:", payload)
        sendNotification({
            // @ts-expect-error
            title: `arsd confirmation - ${payload.user_code || "Unknown"}`,
            body: `Check the confirmation code to start your ${partition} session`,
            channelId: "arsd"
        });
    } catch (e) {
        snackbar.value = true
        snackbarMessage.value = `Failed to authenticate for ${partition}: ${e}`
        return
    }
    if ((payload as any).type == "Success") {
        store.$patch({ expires_at: new Date((payload as any).expires_at) });
    } else if ((payload as any).type == "NeedsConfirmation") {
        console.log("NeedsConfirmation received", payload)
        store.confirmation = payload as Confirmation;
        await open(store.confirmation.confirmation_url);
        checkToken.value = setInterval(async () => {
            const confirmation: Confirmation = (payload as Confirmation)
            const checkResult = await invoke("check_device_token", {
                tokenEvent: confirmation,
            });
            console.log("checking token", confirmation, checkResult)
            if (checkResult === "Done") {
                console.log("Done, ending timer")
                clearInterval(checkToken.value);
                checkToken.value = null;
            } else if (checkResult === "Pending") {
                console.log("still pending")
            } else {
                console.log("other result", checkResult)
                clearInterval(checkToken.value);
                checkToken.value = null;
            }
        }, store.confirmation.polling_interval * 1000)
    } else {
        console.log("other auth event received", payload)
    }

}
</script>

<template>
    <VSnackbar v-model="snackbar" :timeout="10000" top>
        <VBtn icon="mdi-close" variant="text" @click="snackbar = false" />
        <span>{{ snackbarMessage }}</span>
    </VSnackbar>
    <template v-for="(p, index) in partitions">
        <VDivider v-if="index > 0" />
        <VListItem>
            <VListItemTitle>
                <a :href="p.start_url" target="_blank">{{ p.slug.replace(`${p.region}-`, '') }}</a>
            </VListItemTitle>
            <VListItemSubtitle>
                <template v-if="store.expires_at">
                    Expires in
                    <CountDown :countTo="store.expires_at!" />
                </template>
                <template v-else>
                    Expired
                </template>
            </VListItemSubtitle>
        </VListItem>
        <VListItem link @click="tryAuth(p.slug)">
            <VIcon icon="mdi-refresh" /> Refresh
        </VListItem>
    </template>
</template>
