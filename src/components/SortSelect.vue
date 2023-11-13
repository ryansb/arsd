<script setup lang="ts">
import { useSessionStore } from '../store';

const store = useSessionStore();
</script>

<template>
    <VBtnToggle rounded="pill" color="primary" v-model="store.sort" mandatory>
        <VBtn>
            <VIcon>mdi-sort-alphabetical-ascending</VIcon>
            <VTooltip activator="parent" location="bottom" open-delay="500">Sort accounts by name</VTooltip>
        </VBtn>
        <VBtn>
            <VIcon>mdi-chart-histogram</VIcon>
            <VTooltip activator="parent" location="bottom" open-delay="500">Sort accounts by frequency score</VTooltip>
        </VBtn>
    </VBtnToggle>
</template>