<script setup lang="ts">
export type ToolIconName =
  | "audio"
  | "cover"
  | "entrance"
  | "frame"
  | "pip"
  | "adjust"
  | "fusion"
  | "rotate"
  | "speed"
  | "zoom"
  | "file"
  | "folder"
  | "folder-open"
  | "trash"
  | "video"
  | "settings"
  | "info"
  | "check"
  | "chevron";

defineProps<{
  name: ToolIconName;
}>();
</script>

<template>
  <svg class="tool-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <template v-if="name === 'audio'">
      <path d="M4 9v6" />
      <path d="M8 6v12" />
      <path d="M12 3v18" />
      <path d="M16 7v10" />
      <path d="M20 10v4" />
    </template>
    <template v-else-if="name === 'cover'">
      <rect x="3.5" y="4" width="17" height="16" rx="2" />
      <circle cx="8.5" cy="9" r="1.3" />
      <path d="m5.5 17 4.2-4 3.1 2.6 2.1-2 3.6 3.4" />
    </template>
    <template v-else-if="name === 'entrance'">
      <path d="M4 18h16" />
      <path d="M7 15V7" />
      <path d="m4 10 3-3 3 3" />
      <path d="M13 7h7" />
      <path d="M13 11h5" />
      <path d="M13 15h3" />
    </template>
    <template v-else-if="name === 'frame'">
      <path d="M4 7V4h3M17 4h3v3M20 17v3h-3M7 20H4v-3" />
      <path d="M8 12h8" />
      <path d="m12 8 4 4-4 4" />
    </template>
    <template v-else-if="name === 'pip'">
      <rect x="3.5" y="4" width="17" height="16" rx="2" />
      <rect x="12.5" y="12" width="5" height="4" rx=".8" />
    </template>
    <template v-else-if="name === 'adjust'">
      <path d="M5 5v14M12 5v14M19 5v14" />
      <path d="M3 8h4M10 15h4M17 10h4" />
      <circle cx="5" cy="8" r="1.6" fill="currentColor" stroke="none" />
      <circle cx="12" cy="15" r="1.6" fill="currentColor" stroke="none" />
      <circle cx="19" cy="10" r="1.6" fill="currentColor" stroke="none" />
    </template>
    <template v-else-if="name === 'fusion'">
      <rect x="4" y="4" width="8" height="8" rx="1.5" />
      <rect x="12" y="12" width="8" height="8" rx="1.5" />
      <path d="m12 8 4 4M8 12l4 4" />
    </template>
    <template v-else-if="name === 'rotate'">
      <path d="M5 9a7 7 0 0 1 12-3l2 2" />
      <path d="M19 4v4h-4" />
      <path d="M19 15a7 7 0 0 1-12 3l-2-2" />
      <path d="M5 20v-4h4" />
    </template>
    <template v-else-if="name === 'speed'">
      <path d="M4 16a8 8 0 1 1 16 0" />
      <path d="m12 12 4-4" />
      <path d="M7 19h10" />
    </template>
    <template v-else-if="name === 'zoom'">
      <path d="M4 9V5a1 1 0 0 1 1-1h4M15 4h3a2 2 0 0 1 2 2v3M20 15v3a2 2 0 0 1-2 2h-3M9 20H6a2 2 0 0 1-2-2v-3" />
      <path d="M8 16 16 8" />
      <path d="M12 8h4v4" />
    </template>
    <template v-else-if="name === 'file'">
      <path d="M7 3.5h7l4 4V20.5H7a2 2 0 0 1-2-2v-13a2 2 0 0 1 2-2Z" />
      <path d="M14 3.5v5h4" />
      <path d="M8.5 13h7M8.5 16h5" />
    </template>
    <template v-else-if="name === 'folder'">
      <path d="M3.5 7.5a2 2 0 0 1 2-2h4l1.8 2h7.2a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-15Z" />
      <path d="M3.5 9.5h17" />
    </template>
    <template v-else-if="name === 'folder-open'">
      <path d="M3.5 7.5a2 2 0 0 1 2-2h4l1.8 2h7.2a2 2 0 0 1 2 2v1h-14Z" />
      <path d="m3.5 10.5 1.2 6.2a2 2 0 0 0 2 1.6h11.1a2 2 0 0 0 1.9-1.5l1.3-5.3H3.5Z" />
    </template>
    <template v-else-if="name === 'trash'">
      <path d="M5 7h14" />
      <path d="M10 4h4l1 3H9l1-3Z" />
      <path d="m7 7 .7 12h8.6L17 7" />
      <path d="M10 10v6M14 10v6" />
    </template>
    <template v-else-if="name === 'video'">
      <rect x="3.5" y="6" width="12" height="12" rx="2" />
      <path d="m15.5 10 5-3v10l-5-3Z" />
    </template>
    <template v-else-if="name === 'settings'">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-1.8 1.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.5v.1h-2.5v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.9.3l-.1.1-1.8-1.8.1-.1A1.7 1.7 0 0 0 8.1 15a1.7 1.7 0 0 0-1.5-1H6.5v-2.5h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.9l-.1-.1 1.8-1.8.1.1a1.7 1.7 0 0 0 1.9.3 1.7 1.7 0 0 0 1-1.5v-.1H15v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.9-.3l.1-.1 1.8 1.8-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.5 1h.1V14h-.1a1.7 1.7 0 0 0-1.5 1Z" />
    </template>
    <template v-else-if="name === 'info'">
      <circle cx="12" cy="12" r="8.5" />
      <path d="M12 10.5v5" />
      <circle cx="12" cy="7.5" r=".7" fill="currentColor" stroke="none" />
    </template>
    <template v-else-if="name === 'check'">
      <path d="m5 12.5 4.2 4.2L19 7" />
    </template>
    <template v-else>
      <path d="m9 5 7 7-7 7" />
    </template>
  </svg>
</template>
