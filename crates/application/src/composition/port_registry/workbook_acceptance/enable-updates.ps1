param([Parameter(Mandatory)][string]$Source,[Parameter(Mandatory)][string]$Destination,[Parameter(Mandatory)][string]$DataSheets,[Parameter(Mandatory)][string]$Field,[Parameter(Mandatory)][string]$Value)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$qaRoot=[IO.Path]::GetFullPath('O:\Project\FOODBALL\tmp\r17-06-monthly-xlsx')
$sourcePath=[IO.Path]::GetFullPath($Source);$destinationPath=[IO.Path]::GetFullPath($Destination)
if([IO.Path]::GetDirectoryName($sourcePath) -ne $qaRoot -or [IO.Path]::GetDirectoryName($destinationPath) -ne $qaRoot -or $sourcePath -eq $destinationPath){throw 'Fixture paths must stay in the explicit QA directory'}
if((Get-Item -LiteralPath $qaRoot -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'QA root must not be a reparse point'}
function Read-XmlEntry($Entry){
 $document=[Xml.XmlDocument]::new();$document.PreserveWhitespace=$true;$document.XmlResolver=$null
 $stream=$Entry.Open();try{$document.Load($stream)}finally{$stream.Dispose()};return ,$document
}
function Namespaces($Document){$manager=[Xml.XmlNamespaceManager]::new($Document.NameTable);$manager.AddNamespace('s','http://schemas.openxmlformats.org/spreadsheetml/2006/main');return ,$manager}
function Column-Rank([string]$Letters){$rank=0;foreach($character in $Letters.ToCharArray()){$rank=$rank*26+[int]$character-64};return $rank}
$mainNs='http://schemas.openxmlformats.org/spreadsheetml/2006/main'
$archiveIn=[IO.Compression.ZipFile]::OpenRead($sourcePath)
$zipOutStream=[IO.File]::Open($destinationPath,[IO.FileMode]::Create,[IO.FileAccess]::ReadWrite)
$archiveOut=[IO.Compression.ZipArchive]::new($zipOutStream,[IO.Compression.ZipArchiveMode]::Create,$false)
$changed=0;$fieldChanged=0
try{
 $stringTable=Read-XmlEntry ($archiveIn.GetEntry('xl/sharedStrings.xml'));$stringNs=Namespaces $stringTable
 $strings=@($stringTable.SelectNodes('/s:sst/s:si',$stringNs)|ForEach-Object {$_.InnerText})
 function Cell-Text($Cell,$Manager){$v=$Cell.SelectSingleNode('s:v',$Manager);if($Cell.GetAttribute('t') -eq 's'){return $strings[[int]$v.InnerText]};if($Cell.GetAttribute('t') -eq 'inlineStr'){return $Cell.SelectSingleNode('s:is',$Manager).InnerText};if($v){return $v.InnerText};return ''}
 $book=Read-XmlEntry ($archiveIn.GetEntry('xl/workbook.xml'));$bookNs=Namespaces $book
 $rels=Read-XmlEntry ($archiveIn.GetEntry('xl/_rels/workbook.xml.rels'));$targets=@{}
 foreach($rel in $rels.DocumentElement.ChildNodes){$targets[$rel.GetAttribute('Id')]=$rel.GetAttribute('Target')}
 $wanted=@($DataSheets -split '\|');$parts=@{}
 foreach($sheet in $book.SelectNodes('/s:workbook/s:sheets/s:sheet',$bookNs)){
  if($sheet.GetAttribute('name') -notin $wanted){continue}
  $id=$sheet.GetAttribute('id','http://schemas.openxmlformats.org/officeDocument/2006/relationships')
  $target=$targets[$id];if(-not $target -or $target.Contains('..')){throw 'Invalid worksheet relation'}
  $part=if($target.StartsWith('/')){$target.TrimStart('/')}else{'xl/'+$target};$parts[$part]=$true
 }
 if($parts.Count -ne $wanted.Count){throw 'Missing parser-identified data sheets'}
 foreach($entry in $archiveIn.Entries){
  $outputEntry=$archiveOut.CreateEntry($entry.FullName,[IO.Compression.CompressionLevel]::Optimal);$outputStream=$outputEntry.Open()
  try{
   if($parts.ContainsKey($entry.FullName)){
    $document=Read-XmlEntry $entry;$ns=Namespaces $document
    $header=$document.SelectSingleNode('/s:worksheet/s:sheetData/s:row[@r="1"]',$ns)
    $column=$null;$fieldColumn=$null
    foreach($cell in $header.SelectNodes('s:c',$ns)){if((Cell-Text $cell $ns) -eq 'action'){$column=$cell.GetAttribute('r') -replace '\d+',''};if((Cell-Text $cell $ns) -eq $Field){$fieldColumn=$cell.GetAttribute('r') -replace '\d+',''}}
    if(-not $column -or -not $fieldColumn){throw 'Data sheet has no requested columns'}
    foreach($row in $document.SelectNodes('/s:worksheet/s:sheetData/s:row',$ns)){
     if($row.GetAttribute('r') -eq '1'){continue}
     $fieldAddress=$fieldColumn+$row.GetAttribute('r')
     if($fieldChanged -eq 0 -and -not $row.SelectSingleNode('s:c[@r="'+$fieldAddress+'"]',$ns)){
      $newCell=$document.CreateElement('c',$mainNs);$newCell.SetAttribute('r',$fieldAddress)
      $nextCell=$null
      foreach($existing in $row.SelectNodes('s:c',$ns)){if((Column-Rank ($existing.GetAttribute('r') -replace '\d+','')) -gt (Column-Rank $fieldColumn)){$nextCell=$existing;break}}
      if($nextCell){$null=$row.InsertBefore($newCell,$nextCell)}else{$null=$row.AppendChild($newCell)}
     }
     foreach($cell in $row.SelectNodes('s:c',$ns)){
      $cellColumn=$cell.GetAttribute('r') -replace '\d+',''
      $isAction=$cellColumn -eq $column -and (Cell-Text $cell $ns) -eq 'skip'
      $isField=$cellColumn -eq $fieldColumn -and $fieldChanged -eq 0
      if(-not $isAction -and -not $isField){continue}
      $replacement=if($isAction){'update'}else{$Value}
      foreach($child in @($cell.ChildNodes)){$null=$cell.RemoveChild($child)}
      $cell.SetAttribute('t','inlineStr');$inline=$document.CreateElement('is',$mainNs);$textNode=$document.CreateElement('t',$mainNs);$textNode.InnerText=$replacement;$null=$inline.AppendChild($textNode);$null=$cell.AppendChild($inline);if($isAction){$changed++}else{$fieldChanged++}
     }
    }
    $document.Save($outputStream)
   }else{$inputStream=$entry.Open();try{$inputStream.CopyTo($outputStream)}finally{$inputStream.Dispose()}}
  }finally{$outputStream.Dispose()}
 }
}finally{$archiveOut.Dispose();$zipOutStream.Dispose();$archiveIn.Dispose()}
if($changed -eq 0 -or $fieldChanged -ne 1){throw 'Expected explicit actions and exactly one changed QA field'}
Write-Output "Explicit QA updates enabled: $changed action cells; exactly one $Field value changed; other cells and workbook entries preserved"
