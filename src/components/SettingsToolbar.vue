<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

const snackbar = ref(false);
const snackbarMessage = ref("Local data cleared");
const configPath = ref("");
const logPath = ref("");

onMounted(async () => {
    // @ts-ignore-next-line
    const { logs, config } = await invoke("storage_path");
    logPath.value = logs;
    configPath.value = config;
});



async function clear() {
    snackbarMessage.value = "Local data cleared"
    await invoke("delete_cache");
    window.location.reload();
}
</script>

<template v-slot="append">
    <VSnackbar v-model="snackbar" :timeout="2000" top>
        <VBtn icon="mdi-close" variant="text" @click="snackbar = false" />
        <span>{{ snackbarMessage }}</span>
    </VSnackbar>
    <VList>
        <VListItem>
            <VListItemTitle>Settings</VListItemTitle>
            <VListItemSubtitle>
                <VIcon icon="mdi-source-branch-sync" /> <a href="https://github.com/ryansb/arsd" target="_blank">
                    Application Repository</a>
            </VListItemSubtitle>
        </VListItem>
        <VDivider />
        <VListItem link
            @click="writeText(configPath); snackbarMessage = 'Config path copied to clipboard'; snackbar = !snackbar">
            <VIcon icon="mdi-file-cog" /> Copy config file path
        </VListItem>
        <VListItem link
            @click="writeText(logPath); snackbarMessage = 'Log file path copied to clipboard'; snackbar = !snackbar">
            <VIcon icon="mdi-file-clock" /> Copy logs path
        </VListItem>
        <VListItem link @click="clear(); snackbar = !snackbar">
            <VIcon icon="mdi-trash-can" />
            Clear Cache
        </VListItem>
    </VList>
</template>
