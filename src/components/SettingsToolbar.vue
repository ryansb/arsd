<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { onMounted } from "vue";


onMounted(async () => {
});

async function clear() {
    await invoke("delete_cache");
    window.location.reload();
}

let snackbar = ref(false);
let snackbarMessage = ref("Local data cleared");
</script>

<template>
    <VSnackbar v-model="snackbar" :timeout="10000" top>
        <VBtn icon="mdi-close" variant="text" @click="snackbar = false" />
        <span>{{ snackbarMessage }}</span>
    </VSnackbar>
    <VListItem link @click="clear(); snackbar = !snackbar">
        <VIcon icon="mdi-trash-can" /> Clear cache
    </VListItem>
</template>
