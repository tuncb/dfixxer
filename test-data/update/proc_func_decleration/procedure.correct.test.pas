unit TestProcedures;

interface

procedure Foo();
function Bar(): Integer;
procedure WithParams(x: Integer);
function WithParamsAndReturn(x: Integer): String;

implementation

procedure Foo();
begin
end;

function Bar(): Integer;
begin
  Result := 42;
end;

procedure WithParams(x: Integer);
begin
end;

function WithParamsAndReturn(x: Integer): String;
begin
  Result := 'test';
end;

end.