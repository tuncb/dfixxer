unit ContextGatedLtGt;

interface

implementation

function Compare(const A: Integer): Integer;
begin
  if A<10 then
    Result := 1
  else
    Result := 0;
end;

function Broken(const APhases: TArray<Integer>): TArray<Integer>;
begin
  const LData = TList<Integer>.Create();
  for var LPhase in APhases do
  begin
    LData.Add(LPhase);
  end;
  Result := LData.ToArray();
end;

function GenericAfterBoundary: TArray<Integer>;
begin
  Result := nil;
end;

end.
