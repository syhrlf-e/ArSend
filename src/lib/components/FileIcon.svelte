<script lang="ts">
  import {
    File,
    FileText,
    FileImage,
    FileAudio,
    FileVideo,
    FileArchive,
    FileCode
  } from 'lucide-svelte';

  export let filename = '';

  const getFileDetails = (name: string) => {
    const ext = name.split('.').pop()?.toLowerCase() || '';
    switch (ext) {
      case 'pdf':
      case 'doc':
      case 'docx':
      case 'txt':
        return { icon: FileText, color: 'text-blue-500', bg: 'bg-blue-50', border: 'border-blue-100' };
      case 'jpg':
      case 'jpeg':
      case 'png':
      case 'gif':
      case 'webp':
      case 'svg':
        return { icon: FileImage, color: 'text-purple-500', bg: 'bg-purple-50', border: 'border-purple-100' };
      case 'mp3':
      case 'wav':
      case 'ogg':
        return { icon: FileAudio, color: 'text-pink-500', bg: 'bg-pink-50', border: 'border-pink-100' };
      case 'mp4':
      case 'mkv':
      case 'avi':
      case 'mov':
        return { icon: FileVideo, color: 'text-red-500', bg: 'bg-red-50', border: 'border-red-100' };
      case 'zip':
      case 'rar':
      case '7z':
      case 'tar':
      case 'gz':
        return { icon: FileArchive, color: 'text-amber-500', bg: 'bg-amber-50', border: 'border-amber-100' };
      case 'js':
      case 'ts':
      case 'json':
      case 'html':
      case 'css':
      case 'rs':
        return { icon: FileCode, color: 'text-emerald-500', bg: 'bg-emerald-50', border: 'border-emerald-100' };
      default:
        return { icon: File, color: 'text-slate-500', bg: 'bg-slate-100', border: 'border-slate-200' };
    }
  };

  $: details = getFileDetails(filename);
  $: Icon = details.icon;
</script>

<div
  class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border shadow-sm transition-colors {details.bg} {details.border}"
>
  <Icon size={20} class={details.color} strokeWidth={1.5} />
</div>