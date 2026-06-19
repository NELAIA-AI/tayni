$bytes = [System.IO.File]::ReadAllBytes($args[0])
Write-Host "Size:" $bytes.Length
Write-Host "e_lfanew:" ([BitConverter]::ToUInt32($bytes, 60))
$pe = [BitConverter]::ToUInt32($bytes, 60)
Write-Host "PE signature at:" $pe
Write-Host "Machine:" ("0x{0:X}" -f [BitConverter]::ToUInt16($bytes, $pe+4))
Write-Host "NumSections:" ([BitConverter]::ToUInt16($bytes, $pe+6))
Write-Host "OptHdrSize:" ([BitConverter]::ToUInt16($bytes, $pe+20))
Write-Host "EntryPoint:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $pe+40))
Write-Host "ImageBase:" ("0x{0:X}" -f [BitConverter]::ToUInt64($bytes, $pe+48))
Write-Host "SectionAlign:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $pe+56))
Write-Host "FileAlign:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $pe+60))

# Import directory (data dir index 1)
$importDirOffset = $pe + 24 + 112 + 8  # OptHdr start + standard fields + 1 data dir
Write-Host "Import Dir RVA:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $importDirOffset))
Write-Host "Import Dir Size:" ([BitConverter]::ToUInt32($bytes, $importDirOffset+4))

# IAT (data dir index 12)
$iatDirOffset = $pe + 24 + 112 + 96  # OptHdr start + standard fields + 12 data dirs
Write-Host "IAT RVA:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $iatDirOffset))
Write-Host "IAT Size:" ([BitConverter]::ToUInt32($bytes, $iatDirOffset+4))

# Section headers
$secHdrOffset = $pe + 24 + 240  # After COFF + OptHdr
Write-Host "`nSection 0:"
$secName = [System.Text.Encoding]::ASCII.GetString($bytes[$secHdrOffset..($secHdrOffset+7)])
Write-Host "  Name:" $secName
Write-Host "  VirtualSize:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $secHdrOffset+8))
Write-Host "  VirtualAddr:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $secHdrOffset+12))
Write-Host "  RawSize:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $secHdrOffset+16))
Write-Host "  RawPtr:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $secHdrOffset+20))

if ([BitConverter]::ToUInt16($bytes, $pe+6) -gt 1) {
    $secHdrOffset += 40
    Write-Host "`nSection 1:"
    $secName = [System.Text.Encoding]::ASCII.GetString($bytes[$secHdrOffset..($secHdrOffset+7)])
    Write-Host "  Name:" $secName
    Write-Host "  VirtualSize:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $secHdrOffset+8))
    Write-Host "  VirtualAddr:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $secHdrOffset+12))
    Write-Host "  RawSize:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $secHdrOffset+16))
    Write-Host "  RawPtr:" ("0x{0:X}" -f [BitConverter]::ToUInt32($bytes, $secHdrOffset+20))
}
