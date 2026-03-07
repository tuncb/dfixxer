unit InlineSuppression;

interface

// dfixxer:off
uses ZUnit, AUnit, MUnit;
procedure DisabledProc;
// dfixxer:on

procedure EnabledProc();

implementation

uses
  AImpl,
  MImpl,
  ZImpl;

// dfixxer:off
procedure DisabledProc;
begin
  value:=left+right;
end;
// dfixxer:on

procedure EnabledProc();
begin
  value := left + right;
end;

end.
